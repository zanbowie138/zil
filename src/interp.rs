use crate::ast::{BinOp, Expr, ExprKind, FnDef, UnOp};
use crate::lexer::Span;
use crate::Error;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Rc<str>),
    List(Rc<RefCell<Vec<Value>>>),
    Map(Rc<RefCell<IndexMap<String, Value>>>),
    Fn(Rc<Closure>),
    Builtin(&'static str),
}

// ponytail: closures hold their defining Env, so recursive fns form Rc cycles and leak; add a GC/arena if long-running scripts care.
pub struct Closure {
    def: Rc<FnDef>,
    env: Env,
}

impl Value {
    pub fn list(v: Vec<Value>) -> Value {
        Value::List(Rc::new(RefCell::new(v)))
    }

    pub fn map(m: IndexMap<String, Value>) -> Value {
        Value::Map(Rc::new(RefCell::new(m)))
    }

    pub fn str(s: impl Into<Rc<str>>) -> Value {
        Value::Str(s.into())
    }

    pub fn truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Str(_) => "str",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Fn(_) | Value::Builtin(_) => "fn",
        }
    }

    fn write(&self, f: &mut fmt::Formatter, top: bool) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n:?}"),
            Value::Str(s) if top => write!(f, "{s}"),
            Value::Str(s) => write!(f, "{s:?}"),
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.borrow().iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    v.write(f, false)?;
                }
                write!(f, "]")
            }
            Value::Map(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.borrow().iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: ")?;
                    v.write(f, false)?;
                }
                write!(f, "}}")
            }
            Value::Fn(_) => write!(f, "<fn>"),
            Value::Builtin(name) => write!(f, "<builtin {name}>"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write(f, true)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write(f, false)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        use Value::*;
        match (self, other) {
            (Nil, Nil) => true,
            (Bool(a), Bool(b)) => a == b,
            (Int(a), Int(b)) => a == b,
            (Int(_) | Float(_), Int(_) | Float(_)) => num(self) == num(other),
            (Str(a), Str(b)) => a == b,
            (List(a), List(b)) => *a.borrow() == *b.borrow(),
            (Map(a), Map(b)) => *a.borrow() == *b.borrow(),
            (Fn(a), Fn(b)) => Rc::ptr_eq(a, b),
            (Builtin(a), Builtin(b)) => a == b,
            _ => false,
        }
    }
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Int(n) => Some(*n as f64),
        Value::Float(n) => Some(*n),
        _ => None,
    }
}

pub fn compare(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Some(a.cmp(b)),
        (Value::Str(a), Value::Str(b)) => Some(a.cmp(b)),
        _ => num(a)?.partial_cmp(&num(b)?),
    }
}

pub type Env = Rc<RefCell<Scope>>;

#[derive(Default)]
pub struct Scope {
    vars: HashMap<String, Value>,
    parent: Option<Env>,
}

fn child(parent: &Env) -> Env {
    Rc::new(RefCell::new(Scope { vars: HashMap::new(), parent: Some(parent.clone()) }))
}

fn lookup(env: &Env, name: &str) -> Option<Value> {
    let s = env.borrow();
    match s.vars.get(name) {
        Some(v) => Some(v.clone()),
        None => lookup(s.parent.as_ref()?, name),
    }
}

fn assign(env: &Env, name: &str, v: Value) -> bool {
    let mut s = env.borrow_mut();
    if let Some(slot) = s.vars.get_mut(name) {
        *slot = v;
        return true;
    }
    match &s.parent {
        Some(p) => assign(p, name, v),
        None => false,
    }
}

/// Non-local exits threaded through `?`.
enum Ctl {
    Err(Error),
    Return(Value),
}

impl From<Error> for Ctl {
    fn from(e: Error) -> Ctl {
        Ctl::Err(e)
    }
}

type EResult = Result<Value, Ctl>;

pub struct Interp {
    globals: Env,
}

impl Interp {
    pub fn new() -> Interp {
        let globals: Env = Default::default();
        for name in crate::builtins::NAMES {
            globals.borrow_mut().vars.insert(name.to_string(), Value::Builtin(name));
        }
        Interp { globals }
    }

    pub fn run(&mut self, prog: &[Expr]) -> Result<Value, Error> {
        let globals = self.globals.clone();
        let mut last = Value::Nil;
        for stmt in prog {
            last = match self.eval(stmt, &globals) {
                Ok(v) | Err(Ctl::Return(v)) => v,
                Err(Ctl::Err(e)) => return Err(e),
            };
        }
        Ok(last)
    }

    pub fn call(&mut self, f: &Value, args: Vec<Value>, span: &Span) -> Result<Value, Error> {
        match f {
            Value::Fn(c) => {
                if args.len() != c.def.params.len() {
                    return Err(Error::new(
                        format!("expected {} args, got {}", c.def.params.len(), args.len()),
                        span.clone(),
                    ));
                }
                let env = child(&c.env);
                for (p, a) in c.def.params.iter().zip(args) {
                    env.borrow_mut().vars.insert(p.clone(), a);
                }
                match self.eval(&c.def.body, &env) {
                    Ok(v) | Err(Ctl::Return(v)) => Ok(v),
                    Err(Ctl::Err(e)) => Err(e),
                }
            }
            Value::Builtin(name) => crate::builtins::call(self, name, args, span),
            v => Err(Error::new(format!("{} is not callable", v.type_name()), span.clone())),
        }
    }

    fn eval(&mut self, e: &Expr, env: &Env) -> EResult {
        let err = |msg: String| Ctl::Err(Error::new(msg, e.span.clone()));
        Ok(match &e.kind {
            ExprKind::Nil => Value::Nil,
            ExprKind::Bool(b) => Value::Bool(*b),
            ExprKind::Int(n) => Value::Int(*n),
            ExprKind::Float(n) => Value::Float(*n),
            ExprKind::Str(s) => Value::Str(s.clone()),
            ExprKind::Ident(name) => lookup(env, name).ok_or_else(|| err(format!("undefined variable `{name}`")))?,
            ExprKind::List(items) => {
                let mut v = Vec::with_capacity(items.len());
                for it in items {
                    v.push(self.eval(it, env)?);
                }
                Value::list(v)
            }
            ExprKind::Map(entries) => {
                let mut m = IndexMap::new();
                for (k, v) in entries {
                    m.insert(k.clone(), self.eval(v, env)?);
                }
                Value::map(m)
            }
            ExprKind::Unary(op, x) => match (op, self.eval(x, env)?) {
                (UnOp::Not, v) => Value::Bool(!v.truthy()),
                (UnOp::Neg, Value::Int(n)) => Value::Int(n.checked_neg().ok_or_else(|| err("integer overflow".into()))?),
                (UnOp::Neg, Value::Float(n)) => Value::Float(-n),
                (UnOp::Neg, v) => return Err(err(format!("cannot negate {}", v.type_name()))),
            },
            ExprKind::Binary(BinOp::And, a, b) => {
                let a = self.eval(a, env)?;
                if a.truthy() { self.eval(b, env)? } else { a }
            }
            ExprKind::Binary(BinOp::Or, a, b) => {
                let a = self.eval(a, env)?;
                if a.truthy() { a } else { self.eval(b, env)? }
            }
            ExprKind::Binary(op, a, b) => {
                let (a, b) = (self.eval(a, env)?, self.eval(b, env)?);
                binary(*op, &a, &b).map_err(err)?
            }
            ExprKind::Let(name, v) => {
                let v = self.eval(v, env)?;
                env.borrow_mut().vars.insert(name.clone(), v);
                Value::Nil
            }
            ExprKind::Assign(target, v) => {
                let v = self.eval(v, env)?;
                match &target.kind {
                    ExprKind::Ident(name) => {
                        if !assign(env, name, v.clone()) {
                            return Err(err(format!("undefined variable `{name}` (use `let`)")));
                        }
                    }
                    ExprKind::Field(obj, key) => match self.eval(obj, env)? {
                        Value::Map(m) => {
                            m.borrow_mut().insert(key.clone(), v.clone());
                        }
                        o => return Err(err(format!("cannot set field on {}", o.type_name()))),
                    },
                    ExprKind::Index(obj, idx) => {
                        let (obj, idx) = (self.eval(obj, env)?, self.eval(idx, env)?);
                        match (obj, idx) {
                            (Value::List(l), Value::Int(i)) => {
                                let mut l = l.borrow_mut();
                                let i = list_index(i, l.len()).ok_or_else(|| err("index out of range".into()))?;
                                l[i] = v.clone();
                            }
                            (Value::Map(m), Value::Str(k)) => {
                                m.borrow_mut().insert(k.to_string(), v.clone());
                            }
                            (o, i) => return Err(err(format!("cannot index {} with {}", o.type_name(), i.type_name()))),
                        }
                    }
                    _ => unreachable!("parser validates assignment targets"),
                }
                v
            }
            ExprKind::Call(callee, args) => {
                // `x.f(a)`: a map's own fn field wins, else sugar for `f(x, a)`.
                let (f, mut argv) = match &callee.kind {
                    ExprKind::Field(obj, name) => {
                        let o = self.eval(obj, env)?;
                        let own = match &o {
                            Value::Map(m) => m.borrow().get(name).cloned(),
                            _ => None,
                        };
                        match own {
                            Some(f) => (f, vec![]),
                            None => (lookup(env, name).ok_or_else(|| err(format!("no method `{name}`")))?, vec![o]),
                        }
                    }
                    _ => (self.eval(callee, env)?, vec![]),
                };
                for a in args {
                    argv.push(self.eval(a, env)?);
                }
                self.call(&f, argv, &e.span)?
            }
            ExprKind::Field(obj, key) => match self.eval(obj, env)? {
                Value::Map(m) => m.borrow().get(key).cloned().unwrap_or(Value::Nil),
                o => return Err(err(format!("{} has no fields", o.type_name()))),
            },
            ExprKind::Index(obj, idx) => match (self.eval(obj, env)?, self.eval(idx, env)?) {
                (Value::List(l), Value::Int(i)) => {
                    let l = l.borrow();
                    let i = list_index(i, l.len()).ok_or_else(|| err("index out of range".into()))?;
                    l[i].clone()
                }
                (Value::Map(m), Value::Str(k)) => m.borrow().get(&*k).cloned().unwrap_or(Value::Nil),
                (o, i) => return Err(err(format!("cannot index {} with {}", o.type_name(), i.type_name()))),
            },
            ExprKind::Fn(def) => Value::Fn(Rc::new(Closure { def: def.clone(), env: env.clone() })),
            ExprKind::If(cond, then, els) => {
                if self.eval(cond, env)?.truthy() {
                    self.eval(then, env)?
                } else if let Some(els) = els {
                    self.eval(els, env)?
                } else {
                    Value::Nil
                }
            }
            ExprKind::While(cond, body) => {
                while self.eval(cond, env)?.truthy() {
                    self.eval(body, env)?;
                }
                Value::Nil
            }
            ExprKind::For(name, iter, body) => {
                let items: Vec<Value> = match self.eval(iter, env)? {
                    Value::List(l) => l.borrow().clone(),
                    Value::Map(m) => m.borrow().keys().map(|k| Value::str(k.as_str())).collect(),
                    Value::Str(s) => s.chars().map(|c| Value::str(c.to_string())).collect(),
                    v => return Err(err(format!("cannot iterate {}", v.type_name()))),
                };
                for it in items {
                    let scope = child(env);
                    scope.borrow_mut().vars.insert(name.clone(), it);
                    self.eval(body, &scope)?;
                }
                Value::Nil
            }
            ExprKind::Block(stmts) => {
                let scope = child(env);
                let mut last = Value::Nil;
                for s in stmts {
                    last = self.eval(s, &scope)?;
                }
                last
            }
            ExprKind::Return(v) => {
                let v = match v {
                    Some(v) => self.eval(v, env)?,
                    None => Value::Nil,
                };
                return Err(Ctl::Return(v));
            }
        })
    }
}

/// Negative indices count from the end.
fn list_index(i: i64, len: usize) -> Option<usize> {
    let i = if i < 0 { len as i64 + i } else { i };
    (0..len as i64).contains(&i).then_some(i as usize)
}

fn binary(op: BinOp, a: &Value, b: &Value) -> Result<Value, String> {
    use Value::*;
    Ok(match (op, a, b) {
        (BinOp::Eq, _, _) => Bool(a == b),
        (BinOp::Ne, _, _) => Bool(a != b),
        (BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge, _, _) => {
            let ord = compare(a, b).ok_or_else(|| format!("cannot compare {} and {}", a.type_name(), b.type_name()))?;
            Bool(match op {
                BinOp::Lt => ord.is_lt(),
                BinOp::Le => ord.is_le(),
                BinOp::Gt => ord.is_gt(),
                _ => ord.is_ge(),
            })
        }
        (BinOp::Add, Str(_), _) | (BinOp::Add, _, Str(_)) => Value::str(format!("{a}{b}")),
        (BinOp::Add, List(x), List(y)) => Value::list(x.borrow().iter().chain(y.borrow().iter()).cloned().collect()),
        (_, Int(x), Int(y)) => {
            let r = match op {
                BinOp::Add => x.checked_add(*y),
                BinOp::Sub => x.checked_sub(*y),
                BinOp::Mul => x.checked_mul(*y),
                BinOp::Div => x.checked_div(*y),
                _ => x.checked_rem(*y),
            };
            Int(r.ok_or(if *y == 0 { "division by zero" } else { "integer overflow" })?)
        }
        _ => match (num(a), num(b)) {
            (Some(x), Some(y)) => Float(match op {
                BinOp::Add => x + y,
                BinOp::Sub => x - y,
                BinOp::Mul => x * y,
                BinOp::Div => x / y,
                _ => x % y,
            }),
            _ => return Err(format!("unsupported operands {} and {}", a.type_name(), b.type_name())),
        },
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn eval(src: &str) -> Value {
        let ast = crate::parser::parse(crate::lexer::lex(src).unwrap()).unwrap();
        Interp::new().run(&ast).unwrap_or_else(|e| panic!("{}", e.msg))
    }

    #[test]
    fn arithmetic() {
        assert_eq!(eval("1 + 2 * 3"), Value::Int(7));
        assert_eq!(eval("7 / 2.0"), Value::Float(3.5));
        assert_eq!(eval(r#""a" + 1"#), Value::str("a1"));
    }

    #[test]
    fn closures_capture() {
        let src = "
let make = fn(n) { \\x -> x + n }
let add2 = make(2)
add2(40)";
        assert_eq!(eval(src), Value::Int(42));
    }

    #[test]
    fn recursion_and_return() {
        let src = "
let fib = fn(n) {
  if n < 2 { return n }
  fib(n - 1) + fib(n - 2)
}
fib(10)";
        assert_eq!(eval(src), Value::Int(55));
    }

    #[test]
    fn records_and_mutation() {
        let src = "
let u = {name: \"ann\", tags: []}
u.age = 30
u.tags.push(\"x\")
u.tags[0] = \"y\"
[u.age, u.tags, u.missing]";
        assert_eq!(eval(src).to_string(), r#"[30, ["y"], nil]"#);
    }

    #[test]
    fn pipe_and_method_chain_agree() {
        let a = eval("[1, 2, 3, 4]\n  |> filter(\\x -> x % 2 == 0)\n  |> map(\\x -> x * 10)");
        let b = eval("[1, 2, 3, 4].filter(\\x -> x % 2 == 0).map(\\x -> x * 10)");
        assert_eq!(a, b);
        assert_eq!(a.to_string(), "[20, 40]");
    }

    #[test]
    fn division_by_zero_errors() {
        let ast = crate::parser::parse(crate::lexer::lex("1 / 0").unwrap()).unwrap();
        assert!(Interp::new().run(&ast).is_err());
    }
}
