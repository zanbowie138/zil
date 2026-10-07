use crate::Error;
use crate::ast::{BinOp, Expr, ExprKind, FnDef, Pat, Radix, Target, UnOp};
use crate::lexer::Span;
use crate::modules::{self, math::uncertainty, units};
use crate::value::{Value, compare, exact, num, ratio};
use indexmap::IndexMap;
use num_bigint::BigInt;
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ponytail: closures hold their defining Env, so recursive fns form Rc cycles and leak; add a GC/arena if long-running scripts care.
pub struct Closure {
    def: Rc<FnDef>,
    env: Env,
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

pub fn lookup(env: &Env, name: &str) -> Option<Value> {
    let s = env.borrow();
    match s.vars.get(name) {
        Some(v) => Some(v.clone()),
        None => lookup(s.parent.as_ref()?, name),
    }
}

/// Update an existing variable in the nearest scope that has it.
fn assign(env: &Env, name: &str, v: Value) -> Result<(), Value> {
    let mut s = env.borrow_mut();
    if let Some(slot) = s.vars.get_mut(name) {
        *slot = v;
        return Ok(());
    }
    match &s.parent {
        Some(p) => assign(p, name, v),
        None => Err(v),
    }
}

/// Binds `v` to `pat`: `define` makes new names in `env` (params, loop variables), else names follow `=`.
fn bind(pat: &Pat, v: Value, env: &Env, define: bool) -> Result<(), Error> {
    match pat {
        Pat::Skip => {}
        Pat::Name(name) if define => {
            env.borrow_mut().vars.insert(name.clone(), v);
        }
        Pat::Name(name) => {
            if let Err(v) = assign(env, name, v) {
                env.borrow_mut().vars.insert(name.clone(), v);
            }
        }
        Pat::List(pats, span) => {
            let Value::List(l) = &v else {
                return Err(
                    Error::new(format!("expected a list to unpack, got {}", v.type_name()), span.clone()).note(format!("the value is {}", modules::short(&v)))
                );
            };
            let items = l.borrow().clone();
            if items.len() != pats.len() {
                return Err(Error::new(format!("expected {} items to unpack, got {}", pats.len(), items.len()), span.clone())
                    .note(format!("the value is {}", modules::short(&v))));
            }
            for (p, it) in pats.iter().zip(items) {
                bind(p, it, env, define)?;
            }
        }
    }
    Ok(())
}

/// Non-local exits threaded through `?`.
enum Ctl {
    Err(Error),
    Return(Value),
    Break(Span),
    Continue(Span),
}

impl Ctl {
    /// What escapes a function body or the top level: the returned value, or an error.
    fn finish(self) -> Result<Value, Error> {
        match self {
            Ctl::Return(v) => Ok(v),
            Ctl::Err(e) => Err(e),
            Ctl::Break(s) => Err(Error::new("`break` outside a loop", s)),
            Ctl::Continue(s) => Err(Error::new("`continue` outside a loop", s)),
        }
    }
}

impl From<Error> for Ctl {
    fn from(e: Error) -> Ctl {
        Ctl::Err(e)
    }
}

type EResult = Result<Value, Ctl>;

pub struct Interp {
    globals: Env,
    /// Stdin, read on first use of `input`.
    pub input: Option<Rc<str>>,
    /// Nested user fn calls, capped at `MAX_DEPTH` so runaway recursion errors instead of overflowing.
    depth: usize,
}

/// Fits in `main::STACK` with room to spare in release; debug frames are ~10x bigger, so heavy bodies can still overflow there.
const MAX_DEPTH: usize = 10_000;

impl Interp {
    pub fn new() -> Interp {
        let globals: Env = Default::default();
        {
            let mut g = globals.borrow_mut();
            for m in modules::modules() {
                for f in m.fns {
                    g.vars.insert(f.name.to_string(), Value::Builtin(m, f.name));
                }
                for (name, x) in m.consts {
                    g.vars.insert(name.to_string(), Value::Float(*x));
                }
                for (name, x, terms) in m.quantities {
                    g.vars.insert(name.to_string(), Value::Qty(*x, units::terms(terms).expect("checked by modules::tests::registry")));
                }
            }
        }
        Interp { globals, input: None, depth: 0 }
    }

    /// Shared handle to the global scope, so the REPL completer can peek at variables.
    pub fn globals(&self) -> Env {
        self.globals.clone()
    }

    pub fn set_global(&mut self, name: &str, v: Value) {
        self.globals.borrow_mut().vars.insert(name.into(), v);
    }

    pub fn run(&mut self, prog: &[Expr]) -> Result<Value, Error> {
        let globals = self.globals.clone();
        let mut last = Value::Nil;
        for stmt in prog {
            last = self.eval(stmt, &globals).or_else(Ctl::finish)?;
        }
        Ok(last)
    }

    pub fn call(&mut self, f: &Value, args: Vec<Value>, span: &Span) -> Result<Value, Error> {
        self.call_at(f, args, span, &[], None)
    }

    /// `call` with each argument's span and the callee's name as written, for sharper errors.
    fn call_at(&mut self, f: &Value, args: Vec<Value>, span: &Span, arg_spans: &[Span], name: Option<&str>) -> Result<Value, Error> {
        match f {
            Value::Fn(c) => {
                let n = c.def.params.len();
                if args.len() != n {
                    let who = name.unwrap_or("the function");
                    let at = arg_spans.get(n).unwrap_or(span).clone();
                    let mut e = Error::new(format!("{who} takes {}, got {}", modules::args_word(n), args.len()), at);
                    for s in arg_spans.iter().skip(n) {
                        e = e.label(s.clone(), "extra argument");
                    }
                    return Err(e);
                }
                if self.depth >= MAX_DEPTH {
                    return Err(Error::new("recursion too deep", span.clone()));
                }
                let env = child(&c.env);
                for (p, a) in c.def.params.iter().zip(args) {
                    bind(p, a, &env, true)?;
                }
                self.depth += 1;
                let r = self.eval(&c.def.body, &env).or_else(Ctl::finish);
                self.depth -= 1;
                r
            }
            Value::Builtin(m, name) => self.call_builtin(m, name, args, span, arg_spans),
            v => Err(Error::new(format!("cannot call a {}", v.type_name()), span.clone()).note(format!("{} is {}", name.unwrap_or("it"), modules::short(v)))),
        }
    }

    /// Out of `call_at` so its locals don't grow every recursion level.
    #[inline(never)]
    fn call_builtin(&mut self, m: &'static modules::Module, name: &'static str, args: Vec<Value>, span: &Span, arg_spans: &[Span]) -> Result<Value, Error> {
        let doc = m.fns.iter().find(|f| f.name == name).expect("builtins have docs");
        let mut call = |args: &[Value]| (m.call)(self, name, args, span);
        let r = if args.iter().any(|a| matches!(a, Value::Unc(..))) && uncertainty::LIFTED.contains(&name) {
            uncertainty::propagate(&args, call)
        } else {
            call(&args)
        };
        r.map_err(|f| f.error(doc, &args, span, arg_spans))
    }

    /// `obj.key`: a map's key, else a zero-arg method (`s.upper` is `upper(s)`), else nil for maps.
    /// The second half is the keys of a map that just lacked `key`, so `{a: 1}.b.c` can blame `.b`.
    fn field(&mut self, e: &Expr, obj: &Expr, key: &str, env: &Env) -> Result<(Value, Option<Vec<String>>), Ctl> {
        let (o, missed) = match &obj.kind {
            ExprKind::Field(inner, k) => self.field(obj, inner, k, env)?,
            _ => (self.eval(obj, env)?, None),
        };
        let missing_key = |keys: Vec<String>| {
            let ExprKind::Field(_, k) = &obj.kind else { unreachable!("only fields miss keys") };
            let at = obj.span.end - k.len()..obj.span.end;
            let hint = crate::error::did_you_mean(k, keys.iter().map(String::as_str));
            Ctl::Err(Error::new(format!("map has no key `{k}`{hint}"), at))
        };
        if let Value::Map(m) = &o
            && let Some(v) = m.borrow().get(key)
        {
            return Ok((v.clone(), None));
        }
        match (lookup(env, key), missed) {
            (Some(f @ (Value::Fn(_) | Value::Builtin(..))), missed) => match self.call_at(&f, vec![o], &e.span, std::slice::from_ref(&obj.span), Some(key)) {
                Ok(v) => Ok((v, None)),
                Err(_) if missed.is_some() => Err(missing_key(missed.unwrap())),
                Err(err) => Err(err.into()),
            },
            (_, _) if let Value::Map(m) = &o => Ok((Value::Nil, Some(m.borrow().keys().cloned().collect()))),
            (_, Some(keys)) => Err(missing_key(keys)),
            _ => {
                let at = e.span.end - key.len()..e.span.end;
                let hint = crate::error::did_you_mean(key, methods(env).iter().map(String::as_str));
                Err(Ctl::Err(
                    Error::new(format!("{} has no field or method `{key}`{hint}", o.type_name()), at)
                        .label(obj.span.clone(), format!("this is {}", modules::short(&o))),
                ))
            }
        }
    }

    /// `x to target` for unit and string targets. Kept out of `eval` (like `call_expr`) so its locals don't grow every recursion level.
    #[inline(never)]
    fn convert_to(&mut self, e: &Expr, x: &Expr, target: &Target, env: &Env) -> EResult {
        let v = self.eval(x, env)?;
        match modules::convert(&v, target) {
            Some(Ok(r)) => Ok(r),
            Some(Err(msg)) => {
                // Name both sides as written: `5 kmh to kg` says kmh, not kph.
                let msg = match (&v, target) {
                    (Value::Qty(_, u), Target::Unit(spec)) if units::unit_of(spec).is_ok_and(|t| t.dim() != u.dim()) => {
                        let t = units::unit_of(spec).unwrap();
                        format!("cannot convert {} to {}", units::describe(u, written(x).as_deref()), units::describe(&t, Some(&units::spec_name(spec))))
                    }
                    _ => msg,
                };
                if msg.starts_with("unknown unit") {
                    // ponytail: assumes the target is written compactly (`km/h`, not `km / h`).
                    let len = target_name(target).len() - 2;
                    return Err(Ctl::Err(Error::new(msg, e.span.end.saturating_sub(len)..e.span.end)));
                }
                let e = Error::new(msg, x.span.clone());
                Err(Ctl::Err(if written(x).is_some() { e } else { e.label(x.span.clone(), format!("this is {}", modules::short(&v))) }))
            }
            None => Err(Ctl::Err(Error::new(format!("cannot convert {} to {}", v.type_name(), target_name(target)), e.span.clone()))),
        }
    }

    /// `f(a)` and `x.f(a)`: a map's own fn field wins, else sugar for `f(x, a)`.
    #[inline(never)]
    fn call_expr(&mut self, e: &Expr, callee: &Expr, args: &[Expr], env: &Env) -> EResult {
        let (f, mut argv) = match &callee.kind {
            ExprKind::Field(obj, name) => {
                let o = self.eval(obj, env)?;
                let own = match &o {
                    Value::Map(m) => m.borrow().get(name).cloned(),
                    _ => None,
                };
                match own {
                    Some(f) => (f, vec![]),
                    None => {
                        let Some(f) = lookup(env, name) else {
                            let at = callee.span.end - name.len()..callee.span.end;
                            let hint = crate::error::did_you_mean(name, methods(env).iter().map(String::as_str));
                            return Err(Ctl::Err(Error::new(format!("{} has no method `{name}`{hint}", o.type_name()), at)));
                        };
                        (f, vec![o])
                    }
                }
            }
            _ => (self.eval(callee, env)?, vec![]),
        };
        let mut spans: Vec<Span> = match &callee.kind {
            ExprKind::Field(obj, _) if !argv.is_empty() => vec![obj.span.clone()],
            _ => vec![],
        };
        for a in args {
            argv.push(self.eval(a, env)?);
            spans.push(a.span.clone());
        }
        let name = match &callee.kind {
            ExprKind::Ident(n) | ExprKind::Field(_, n) => Some(n.as_str()),
            _ => None,
        };
        Ok(self.call_at(&f, argv, &e.span, &spans, name)?)
    }

    /// `unit pizza`, `unit slice = pizza / 8`; out of `eval` to keep its frame small.
    #[inline(never)]
    fn unit_def(&mut self, name: &str, value: Option<&Expr>, env: &Env, span: &Span) -> EResult {
        let v = match value {
            Some(x) => Some(self.eval(x, env)?),
            None => None,
        };
        units::define(name, v.as_ref()).map_err(|m| Ctl::Err(Error::new(format!("unit: {m}"), span.clone())))?;
        Ok(Value::Nil)
    }

    fn eval(&mut self, e: &Expr, env: &Env) -> EResult {
        let err = |msg: String| Ctl::Err(Error::new(msg, e.span.clone()));
        Ok(match &e.kind {
            ExprKind::Nil => Value::Nil,
            ExprKind::Bool(b) => Value::Bool(*b),
            ExprKind::Int(n, b) => Value::Int(*n, Radix { base: *b, width: 0 }),
            ExprKind::Big(n, b) => Value::Big(n.clone(), Radix { base: *b, width: 0 }),
            ExprKind::Float(n) => Value::Float(*n),
            ExprKind::Dec(r) => Value::Frac(r.clone(), false),
            ExprKind::Str(s) => Value::Str(s.clone()),
            ExprKind::Regex(r) => Value::Regex(r.clone()),
            ExprKind::Ident(name) => match lookup(env, name) {
                Some(v) => v,
                None => modules::ident(self, name)
                    .unwrap_or_else(|| Err(format!("undefined variable `{name}`{}", crate::error::did_you_mean(name, names(env).iter().map(String::as_str)))))
                    .map_err(err)?,
            },
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
            ExprKind::Qty(n, spec) => {
                let v = self.eval(n, env)?;
                let n = num(&v).ok_or_else(|| {
                    Ctl::Err(
                        Error::new(format!("cannot attach a unit to a {}", v.type_name()), n.span.clone())
                            .label(n.span.clone(), format!("this is {}", modules::short(&v))),
                    )
                })?;
                Value::Qty(n, units::unit_of(spec).map_err(err)?)
            }
            // `x to hex(8)` is the builtin call `hex(x, 8)`, even if `hex` is shadowed.
            ExprKind::To(v, Target::Named(name, arg)) => {
                let (m, f) = modules::target(name).expect("parser only accepts registered targets");
                let mut args = vec![self.eval(v, env)?];
                args.extend(arg.map(Value::int));
                self.call(&Value::Builtin(m, f), args, &e.span)?
            }
            ExprKind::To(x, target) => self.convert_to(e, x, target, env)?,
            ExprKind::Percent(x) => binary(BinOp::Div, &self.eval(x, env)?, &Value::int(100)).map_err(err)?,
            ExprKind::Format(x, spec) => {
                let v = self.eval(x, env)?;
                let spec = modules::math::formatting::Spec::parse(spec).expect("parser checks specs");
                Value::str(modules::math::formatting::render(&v, &spec).map_err(err)?)
            }
            ExprKind::Unary(op, x) => match (op, self.eval(x, env)?) {
                (UnOp::Not, v) => Value::Bool(!v.truthy()),
                (UnOp::Neg, Value::Int(n, b)) => n.checked_neg().map_or_else(|| Value::Big(Rc::new(-BigInt::from(n)), b), |n| Value::Int(n, b)),
                (UnOp::Neg, Value::Big(n, b)) => exact(BigRational::from_integer(-&*n), b, false),
                (UnOp::Neg, Value::Frac(r, f)) => Value::Frac(Rc::new(-&*r), f),
                (UnOp::Neg, Value::Float(n)) => Value::Float(-n),
                (UnOp::Neg, Value::Qty(n, u)) => Value::Qty(-n, u),
                (UnOp::Neg, Value::Unc(c)) => Value::unc(-c.0, c.1, c.2.clone()),
                (UnOp::Neg, v) => return Err(err(format!("cannot negate a {}", v.type_name()))),
                (UnOp::BitNot, Value::Int(n, b)) => Value::Int(!n, b),
                (UnOp::BitNot, Value::Big(n, b)) => exact(BigRational::from_integer(!&*n), b, false),
                (UnOp::BitNot, v) => {
                    return Err(err(format!("cannot bit-invert a {}", v.type_name())));
                }
            },
            ExprKind::Binary(BinOp::And, a, b) => {
                let a = self.eval(a, env)?;
                if a.truthy() { self.eval(b, env)? } else { a }
            }
            ExprKind::Binary(BinOp::Or, a, b) => {
                let a = self.eval(a, env)?;
                if a.truthy() { a } else { self.eval(b, env)? }
            }
            // `x in v` is `contains(v, x)`, even if `contains` is shadowed; `5 km in mi` is `to`.
            ExprKind::Binary(BinOp::In, x, v) => match (self.eval(x, env)?, self.eval(v, env)?) {
                (Value::Qty(x, u), Value::Qty(_, t)) => units::to_unit(x, &u, t).map_err(err)?,
                (xv, vv) => self.call_at(&modules::builtin("contains"), vec![vv, xv], &e.span, &[v.span.clone(), x.span.clone()], None)?,
            },
            ExprKind::Chain(first, rest) => {
                let (mut l, mut le) = (self.eval(first, env)?, &**first);
                for (op, r) in rest {
                    let rv = self.eval(r, env)?;
                    if !binary(*op, &l, &rv).map_err(|m| operands(m, *op, (le, &l), (r, &rv)))?.truthy() {
                        return Ok(Value::Bool(false));
                    }
                    (l, le) = (rv, r);
                }
                Value::Bool(true)
            }
            ExprKind::Binary(op, a, b) => {
                let (av, bv) = (self.eval(a, env)?, self.eval(b, env)?);
                binary(*op, &av, &bv).map_err(|m| operands(m, *op, (a, &av), (b, &bv)))?
            }
            ExprKind::Assign(target, v) => {
                let v = self.eval(v, env)?;
                match &target.kind {
                    ExprKind::Ident(name) => {
                        if let Err(v) = assign(env, name, v.clone()) {
                            env.borrow_mut().vars.insert(name.clone(), v);
                        }
                    }
                    ExprKind::Field(obj, key) => match self.eval(obj, env)? {
                        Value::Map(m) => {
                            m.borrow_mut().insert(key.clone(), v.clone());
                        }
                        o => return Err(err(format!("cannot set a field on a {}", o.type_name()))),
                    },
                    ExprKind::Index(obj, idx) => {
                        let (ov, iv) = (self.eval(obj, env)?, self.eval(idx, env)?);
                        match (ov, iv) {
                            (Value::List(l), Value::Int(i, _)) => {
                                let mut l = l.borrow_mut();
                                let i = list_index(i, l.len()).ok_or_else(|| out_of_range(i, l.len(), obj, idx))?;
                                l[i] = v.clone();
                            }
                            (Value::Map(m), Value::Str(k)) => {
                                m.borrow_mut().insert(k.to_string(), v.clone());
                            }
                            (o, i) => {
                                return Err(err(format!("cannot index {} with {}", o.type_name(), i.type_name())));
                            }
                        }
                    }
                    _ => unreachable!("parser validates assignment targets"),
                }
                v
            }
            ExprKind::Unpack(pat, v) => {
                let v = self.eval(v, env)?;
                bind(pat, v.clone(), env, false)?;
                v
            }
            ExprKind::Call(callee, args) => self.call_expr(e, callee, args, env)?,
            ExprKind::Field(obj, key) => self.field(e, obj, key, env)?.0,
            ExprKind::Index(obj, idx) => match (self.eval(obj, env)?, self.eval(idx, env)?) {
                (Value::List(l), Value::Int(i, _)) => {
                    let l = l.borrow();
                    let at = list_index(i, l.len()).ok_or_else(|| out_of_range(i, l.len(), obj, idx))?;
                    l[at].clone()
                }
                (Value::Str(s), Value::Int(i, _)) => {
                    let n = s.chars().count();
                    let at = list_index(i, n).ok_or_else(|| out_of_range(i, n, obj, idx))?;
                    Value::str(s.chars().nth(at).unwrap().to_string())
                }
                (Value::Map(m), Value::Str(k)) => m.borrow().get(&*k).cloned().unwrap_or(Value::Nil),
                (o, i) => {
                    return Err(err(format!("cannot index {} with {}", o.type_name(), i.type_name())));
                }
            },
            ExprKind::Slice(obj, from, to) => {
                let o = self.eval(obj, env)?;
                let mut bound = |b: &Option<Box<Expr>>| -> Result<Option<i64>, Ctl> {
                    match b {
                        None => Ok(None),
                        Some(b) => match self.eval(b, env)? {
                            Value::Int(n, _) => Ok(Some(n)),
                            v => Err(Ctl::Err(Error::new(format!("slice bounds must be int, got {}", v.type_name()), b.span.clone()))),
                        },
                    }
                };
                let (from, to) = (bound(from)?, bound(to)?);
                match o {
                    Value::List(l) => {
                        let l = l.borrow();
                        let (a, b) = slice_range(from, to, l.len());
                        Value::list(l[a..b].to_vec())
                    }
                    Value::Str(s) => {
                        let (a, b) = slice_range(from, to, s.chars().count());
                        Value::str(s.chars().skip(a).take(b - a).collect::<String>())
                    }
                    o => return Err(err(format!("cannot slice {}", o.type_name()))),
                }
            }
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
                    match self.eval(body, env) {
                        Ok(_) | Err(Ctl::Continue(_)) => {}
                        Err(Ctl::Break(_)) => break,
                        Err(c) => return Err(c),
                    }
                }
                Value::Nil
            }
            ExprKind::For(pat, iter, body) => {
                let items: Vec<Value> = match self.eval(iter, env)? {
                    Value::List(l) => l.borrow().clone(),
                    Value::Map(m) => m.borrow().keys().map(|k| Value::str(k.as_str())).collect(),
                    Value::Set(s) => s.borrow().iter().cloned().collect(),
                    Value::Str(s) => s.chars().map(|c| Value::str(c.to_string())).collect(),
                    v => {
                        return Err(Ctl::Err(
                            Error::new(format!("cannot loop over a {}", v.type_name()), iter.span.clone())
                                .label(iter.span.clone(), format!("this is {}", modules::short(&v))),
                        ));
                    }
                };
                for it in items {
                    let scope = child(env);
                    bind(pat, it, &scope, true)?;
                    match self.eval(body, &scope) {
                        Ok(_) | Err(Ctl::Continue(_)) => {}
                        Err(Ctl::Break(_)) => break,
                        Err(c) => return Err(c),
                    }
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
            ExprKind::UnitDef(name, value) => self.unit_def(name, value.as_deref(), env, &e.span)?,
            ExprKind::Break => return Err(Ctl::Break(e.span.clone())),
            ExprKind::Continue => return Err(Ctl::Continue(e.span.clone())),
        })
    }
}

/// Negative indices count from the end.
fn list_index(i: i64, len: usize) -> Option<usize> {
    let i = if i < 0 { len as i64 + i } else { i };
    (0..len as i64).contains(&i).then_some(i as usize)
}

/// Clamped `[from..to)`, negatives counting from the end.
fn slice_range(from: Option<i64>, to: Option<i64>, len: usize) -> (usize, usize) {
    let fix = |i: i64| (if i < 0 { len as i64 + i } else { i }).clamp(0, len as i64) as usize;
    let a = from.map_or(0, fix);
    let b = to.map_or(len, fix);
    (a, b.max(a))
}

/// Every name in scope, innermost first.
pub fn names(env: &Env) -> Vec<String> {
    let s = env.borrow();
    let mut out: Vec<String> = s.vars.keys().cloned().collect();
    out.extend(s.parent.as_ref().map(names).unwrap_or_default());
    out
}

/// Names in scope that hold functions, for `x.method` suggestions.
pub fn methods(env: &Env) -> Vec<String> {
    names(env).into_iter().filter(|n| matches!(lookup(env, n), Some(Value::Fn(_) | Value::Builtin(..)))).collect()
}

/// A unit as the source wrote it, when `e` is a literal like `5 kmh`.
fn written(e: &Expr) -> Option<String> {
    match &e.kind {
        ExprKind::Qty(_, spec) => Some(units::spec_name(spec)),
        _ => None,
    }
}

fn target_name(t: &Target) -> String {
    match t {
        Target::Unit(spec) => format!("`{}`", units::spec_name(spec)),
        Target::Units(specs) => format!("`{}`", specs.iter().map(units::spec_name).collect::<Vec<_>>().join(" ")),
        Target::Named(n, _) => format!("`{n}`"),
        Target::Str(s) => format!("{s:?}"),
    }
}

/// `index 5 out of range` on the index, with the length in a note.
fn out_of_range(i: i64, len: usize, obj: &Expr, idx: &Expr) -> Ctl {
    let what = match &obj.kind {
        ExprKind::Ident(n) => format!("`{n}`"),
        _ => "it".into(),
    };
    let items = if matches!(obj.kind, ExprKind::Str(_)) { "character" } else { "item" };
    Ctl::Err(Error::new(format!("index {i} out of range"), idx.span.clone()).note(format!("{what} has {len} {items}{}", if len == 1 { "" } else { "s" })))
}

/// A binary operator's error, labeling each operand with its value. Mismatched quantities are named
/// by kind and unit as written: `cannot add length (km) and mass (kg)`.
fn operands(msg: String, op: BinOp, (a, av): (&Expr, &Value), (b, bv): (&Expr, &Value)) -> Ctl {
    let msg = match (av, bv) {
        (Value::Qty(_, u), Value::Qty(_, w)) if u.dim() != w.dim() => {
            format!("cannot {} {} and {}", verb(op), units::describe(u, written(a).as_deref()), units::describe(w, written(b).as_deref()))
        }
        _ => msg,
    };
    Ctl::Err(Error::new(msg, a.span.clone()).label(a.span.clone(), modules::short(av)).label(b.span.clone(), modules::short(bv)))
}

fn verb(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "add",
        BinOp::Sub => "subtract",
        BinOp::PlusMinus => "attach an uncertainty to",
        BinOp::Mul => "multiply",
        BinOp::Div | BinOp::IntDiv => "divide",
        BinOp::Rem => "take the remainder of",
        BinOp::Pow => "raise",
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => "compare",
        BinOp::Range | BinOp::RangeIncl => "make a range of",
        BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => "bit-combine",
        _ => "combine",
    }
}

/// `int`, or `length (km)` for a quantity.
fn describe(v: &Value) -> String {
    match v {
        Value::Qty(_, u) => units::describe(u, None),
        _ => v.type_name().to_string(),
    }
}

pub fn mismatch(op: BinOp, a: &Value, b: &Value) -> String {
    format!("cannot {} {} and {}", verb(op), describe(a), describe(b))
}

pub fn binary(op: BinOp, a: &Value, b: &Value) -> Result<Value, String> {
    use Value::*;
    let mismatch = || mismatch(op, a, b);
    match op {
        BinOp::Eq => return Ok(Bool(a == b)),
        BinOp::Ne => return Ok(Bool(a != b)),
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            let ord = compare(a, b).ok_or_else(mismatch)?;
            return Ok(Bool(match op {
                BinOp::Lt => ord.is_lt(),
                BinOp::Le => ord.is_le(),
                BinOp::Gt => ord.is_gt(),
                _ => ord.is_ge(),
            }));
        }
        _ => {}
    }
    if let Some(r) = modules::binary(op, a, b) {
        return r;
    }
    if let (Int(x, bx), Int(y, by)) = (a, b)
        && let Some(r) = int_op(op, *x, *y)?
    {
        return Ok(Int(r, if *bx != Radix::DEC { *bx } else { *by }));
    }
    if let (Some(x), Some(y)) = (ratio(a), ratio(b)) {
        return exact_op(op, a, b, x, y);
    }
    Ok(match (num(a), num(b)) {
        (Some(x), Some(y)) => Float(match op {
            BinOp::Add => x + y,
            BinOp::Sub => x - y,
            BinOp::Mul => x * y,
            BinOp::Div => x / y,
            BinOp::IntDiv => (x / y).floor(),
            BinOp::Rem => x.rem_euclid(y),
            BinOp::Pow => x.powf(y),
            _ => return Err(mismatch()),
        }),
        _ => return Err(mismatch()),
    })
}

/// The i64 fast path; `None` hands over to `exact_op` (overflow, uneven division, negative powers).
fn int_op(op: BinOp, x: i64, y: i64) -> Result<Option<i64>, String> {
    Ok(match op {
        BinOp::Add => x.checked_add(y),
        BinOp::Sub => x.checked_sub(y),
        BinOp::Mul => x.checked_mul(y),
        BinOp::Div | BinOp::IntDiv | BinOp::Rem if y == 0 => return Err("division by zero".into()),
        BinOp::Div if x.checked_rem(y) != Some(0) => None,
        BinOp::Div => x.checked_div(y),
        BinOp::IntDiv => x.checked_div(y).map(|q| if x % y != 0 && (x < 0) != (y < 0) { q - 1 } else { q }),
        BinOp::Rem => x.checked_rem_euclid(y),
        BinOp::Pow => u32::try_from(y).ok().and_then(|y| x.checked_pow(y)),
        BinOp::BitAnd => Some(x & y),
        BinOp::BitOr => Some(x | y),
        BinOp::BitXor => Some(x ^ y),
        // ponytail: `<<` drops high bits like C instead of erroring on overflow.
        BinOp::Shl | BinOp::Shr if !(0..64).contains(&y) => return Err(format!("shift by {y} is out of range\nnote: shifts go from 0 to 63")),
        BinOp::Shl => Some(x << y),
        BinOp::Shr => Some(x >> y),
        _ => None,
    })
}

/// Arithmetic on ints, big ints and fractions with no rounding; the result shrinks back to the smallest type.
fn exact_op(op: BinOp, a: &Value, b: &Value, x: BigRational, y: BigRational) -> Result<Value, String> {
    let radix = |v: &Value| match v {
        Value::Int(_, r) | Value::Big(_, r) if *r != Radix::DEC => Some(*r),
        _ => None,
    };
    let base = radix(a).or(radix(b)).unwrap_or(Radix::DEC);
    let as_frac = matches!(a, Value::Frac(_, true)) || matches!(b, Value::Frac(_, true));
    if matches!(op, BinOp::Div | BinOp::IntDiv | BinOp::Rem) && y.is_zero() {
        return Err("division by zero".into());
    }
    let r = match op {
        BinOp::Add => x + y,
        BinOp::Sub => x - y,
        BinOp::Mul => x * y,
        BinOp::Div => x / y,
        BinOp::IntDiv => (x / y).floor(),
        BinOp::Rem => {
            let m = y.abs();
            let q = (&x / &m).floor();
            x - m * q
        }
        BinOp::Pow if !y.is_integer() => {
            return Ok(Value::Float(num(a).unwrap().powf(num(b).unwrap())));
        }
        BinOp::Pow => pow(x, y.to_integer())?,
        BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr if x.is_integer() && y.is_integer() => {
            let (p, q) = (x.to_integer(), y.to_integer());
            BigRational::from_integer(match op {
                BinOp::BitAnd => p & q,
                BinOp::BitOr => p | q,
                BinOp::BitXor => p ^ q,
                _ => {
                    let s = q.to_u8().filter(|s| *s < 64).ok_or(format!("shift by {q} is out of range\nnote: shifts go from 0 to 63"))?;
                    if op == BinOp::Shl { p << s } else { p >> s }
                }
            })
        }
        _ => return Err(mismatch(op, a, b)),
    };
    Ok(exact(r, base, as_frac))
}

/// Exact power, refusing results over ~4M bits (about 1.2M digits).
fn pow(x: BigRational, e: BigInt) -> Result<BigRational, String> {
    if x.is_zero() && e.is_negative() {
        return Err("division by zero".into());
    }
    // 0, 1 and -1 only care whether e is zero, odd or even, however large it is.
    if x.is_zero() || x.abs().is_one() {
        return Ok(x.pow(if e.is_zero() {
            0
        } else if e.is_odd() {
            1
        } else {
            2
        }));
    }
    let bits = x.numer().bits().max(x.denom().bits());
    match e.to_i32() {
        Some(n) if bits.saturating_mul(n.unsigned_abs() as u64) <= 1 << 22 => Ok(x.pow(n)),
        _ => Err("result too large\nnote: exact results stop at about 1.2 million digits; a float base like `2.0 ** n` gives an estimate".into()),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn eval(src: &str) -> Value {
        try_eval(src).unwrap_or_else(|e| panic!("{src}: {}", e.msg))
    }

    pub fn try_eval(src: &str) -> Result<Value, Error> {
        let ast = crate::parser::parse(src)?;
        Interp::new().run(&ast)
    }

    pub fn show(src: &str) -> String {
        eval(src).to_string()
    }

    #[test]
    fn arithmetic() {
        assert_eq!(show("1 + 2 * 3"), "7");
        assert_eq!(show("1 / 3"), "0.333333");
        assert_eq!(show("6 / 3"), "2");
        assert_eq!(show("-7 // 2"), "-4");
        assert_eq!(show("-7 % 3"), "2");
        assert_eq!(show("2 ** 10"), "1024");
        assert_eq!(show("-2 ** 2"), "-4");
        assert_eq!(show("2 ** 3 ** 2"), "512");
        assert_eq!(show("0.1 + 0.2"), "0.3");
        assert_eq!(show("1e20"), "1e20");
        assert_eq!(show(r#""ab" * 3"#), "ababab");
        assert!(try_eval("1 / 0").is_err());
    }

    #[test]
    fn percent() {
        assert_eq!(show("[20%, 50 + 10%, 50 - 10%, 20% of 50, 50 * 10%, 7 % 3, 7%3, 20 % -3]"), "[0.2, 55, 45, 10, 5, 1, 1, 2]");
        assert_eq!(show("[80 km + 25%, 15% of 2 h to min, 1.5%]"), "[100 km, 18 min, 0.015]");
        assert_eq!(
            show(
                "x = 200
x -= 5%
x += 50%
x"
            ),
            "285"
        );
        assert_eq!(show("(100 + 10%) + 10%"), "121");
    }

    #[test]
    fn bitwise() {
        assert_eq!(show("0b1100 & 0b1010 to bin"), "0b1000");
        assert_eq!(show("0b1100 | 0b1010 to bin"), "0b1110");
        assert_eq!(show("0b1100 ^ 0b1010 to bin"), "0b110");
        assert_eq!(show("1 << 4"), "16");
        assert_eq!(show("-16 >> 2"), "-4");
        assert_eq!(show("~0"), "-1");
        assert_eq!(
            show(
                "x = 0xff00 to hex
x & 0x0ff0"
            ),
            "0xf00"
        );
        assert_eq!(show("1 | 2 == 3"), "true");
        assert_eq!(show("0xf0 >> 4 + 0"), "0xf");
        assert!(try_eval("1 << 64").is_err());
        assert!(try_eval("1.5 & 1").is_err());
    }

    #[test]
    fn strings() {
        assert_eq!(
            show(
                r#"x = 2
"x={x}, sq={x ** 2}""#
            ),
            "x=2, sq=4"
        );
        assert_eq!(show(r#""hello"[1..4]"#), "ell");
        assert_eq!(show(r#""hello"[-3..]"#), "llo");
        assert_eq!(show(r#""hello".upper"#), "HELLO");
        assert_eq!(show(r#"" hi ".trim.reverse"#), "ih");
        assert_eq!(show(r#""a,b".split(",").len"#), "2");
    }

    #[test]
    fn variables_and_closures() {
        assert_eq!(show("x = 1\nx = x + 1\nx"), "2");
        let src = "
make = fn(n) { |x| x + n }
add2 = make(2)
add2(40)";
        assert_eq!(show(src), "42");
        assert_eq!(show("f = || 7\nf() + (|a, b| a * b)(2, 3)"), "13");
        let src = "
total = 0
for x in 1..5 { total = total + x }
total";
        assert_eq!(show(src), "10");
    }

    #[test]
    fn recursion_and_return() {
        let src = "
fib = fn(n) {
  if n < 2 { return n }
  fib(n - 1) + fib(n - 2)
}
fib(10)";
        assert_eq!(show(src), "55");
    }

    #[test]
    fn recursion_depth() {
        let run = || {
            let g = "g = |n| if n == 0 {0} else {g(n - 1)}
";
            assert_eq!(show(&format!("{g}g(5000)")), "0");
            assert_eq!(try_eval(&format!("{g}g(10_000_000)")).unwrap_err().msg, "recursion too deep");
        };
        std::thread::Builder::new().stack_size(crate::STACK).spawn(run).unwrap().join().unwrap();
    }

    #[test]
    fn semicolons() {
        assert_eq!(show("x = 1; y = 2; x + y;"), "3");
        assert_eq!(show("f = fn(n) { a = n; a * 2 }; f(4)"), "8");
    }

    #[test]
    fn loop_control() {
        assert_eq!(show("t = 0\nfor x in 1..10 { if x == 5 { break }\n if x % 2 == 0 { continue }\n t += x }\nt"), "4");
        assert_eq!(show("n = 0\nwhile true { n += 1\n if n >= 3 { break } }\nn"), "3");
        assert_eq!(show("f = fn() { for x in [1, 2] { return x } }\nf()"), "1");
        assert!(try_eval("break").is_err());
        assert!(try_eval("for x in [1] { [1].map(|y| continue) }").is_err());
    }

    #[test]
    fn compound_assignment() {
        assert_eq!(show("x = 5\nx += 2\nx *= 3\nx -= 1\nx //= 4\nx **= 2\nx"), "25");
        assert_eq!(show("xs = [1, 2]\nxs[0] += 10\nu = {n: 1}\nu.n <<= 3\n[xs, u.n]"), "[[11, 2], 8]");
        assert_eq!(show("s = \"a\"\ns += \"b\"\ns"), "ab");
        assert!(try_eval("1 += 2").is_err());
    }

    #[test]
    fn pipe_placeholder() {
        assert_eq!(show("3.14159 |> round(_, 2)"), "3.14");
        assert_eq!(show("5 |> _ * 2 |> [_, _]"), "[10, 10]");
        assert_eq!(show("[1, 2] |> map(_, |x| x |> _ + 1)"), "[2, 3]");
        assert_eq!(show("\"x\" |> upper"), "X");
    }

    #[test]
    fn membership_and_chains() {
        assert_eq!(show("[2 in [1, 2], \"b\" in \"abc\", \"k\" in {k: 1}, 9 in 1..5]"), "[true, true, true, false]");
        assert_eq!(show("x = 3\n[x in [3], 2 in 1..=2]"), "[true, true]");
        assert_eq!(show("5 in to cm"), "12.7 cm");
        assert_eq!(show("[5 km in mi, (x = 3 ft) in cm]"), "[3.10686 mi, 91.44 cm]");
        assert!(try_eval("5 km in kg").is_err());
        assert_eq!(show("[0 < 5 < 10, 0 < 15 < 10, 1 <= 1 < 2 <= 2]"), "[true, false, true]");
        assert_eq!(show("n = 0\nf = fn() { n += 1\n 5 }\n0 < f() < 10\nn"), "1");
    }

    #[test]
    fn destructuring() {
        assert_eq!(show("[a, [b, _], c] = [1, [2, 3], 4]\n[a, b, c]"), "[1, 2, 4]");
        assert_eq!(show("a = 1\nb = 2\n[a, b] = [b, a]\n[a, b]"), "[2, 1]");
        assert_eq!(show("t = 0\nfor [k, v] in {x: 1, y: 2}.list { t += v }\nt"), "3");
        assert_eq!(show("s = \"\"\nfor [i, x] in [\"a\", \"b\"].enumerate { s += \"{i}{x}\" }\ns"), "0a1b");
        assert_eq!(show("[[1, 2], [3, 4]].map(|[a, b]| a * b)"), "[2, 12]");
        assert_eq!(show("f = fn([x, y], z) { x + y + z }\nf([1, 2], 3)"), "6");
        assert_eq!(try_eval("[a, b] = [1]").unwrap_err().msg, "expected 2 items to unpack, got 1");
        assert_eq!(try_eval("[a] = 5").unwrap_err().msg, "expected a list to unpack, got int");
        assert!(try_eval("[a, 1] = [1, 2]").is_err());
    }

    #[test]
    fn ranges() {
        assert_eq!(show("1..=4"), "[1, 2, 3, 4]");
        assert_eq!(show("(0..=20).step(5)"), "[0, 5, 10, 15, 20]");
        assert!(try_eval("(1..3).step(0)").is_err());
    }

    #[test]
    fn big_ints() {
        assert_eq!(show("2 ** 100"), "1267650600228229401496703205376");
        assert_eq!(show("9223372036854775807 + 1"), "9223372036854775808");
        assert_eq!(show("(2 ** 64) // 2 ** 60"), "16");
        assert_eq!(type_of("2 ** 64 - 2 ** 64 + 1"), "Int");
        assert_eq!(show("99999999999999999999 % 7"), "1");
        assert_eq!(show("0xffffffffffffffffff + 1"), "0x1000000000000000000");
        assert_eq!(show("(2 ** 100).digits.sum"), "115");
        assert_eq!(show("-(-9223372036854775807 - 1)"), "9223372036854775808");
        assert_eq!(show("\"123456789012345678901234567890\".int"), "123456789012345678901234567890");
        assert_eq!(show("2 ** 64 > 2 ** 63"), "true");
        assert_eq!(show("(-1) ** 99999999999"), "-1");
        assert!(try_eval("2 ** 99999999999").is_err());
    }

    #[test]
    fn fractions() {
        assert_eq!(show("[0.1 + 0.2 == 0.3, 0.1 + 0.2 - 0.3, type(0.1), type(1.0), type(1.5e3)]"), r#"[true, 0, "frac", "float", "float"]"#);
        assert_eq!(show("round(2.675, 2)"), "2.68");
        assert_eq!(show("7 / 2"), "3.5");
        assert_eq!(show("1 / 3 * 3"), "1");
        assert_eq!(show("type(1 / 3)"), "frac");
        assert_eq!(show("1/3 + 1/6 to frac"), "1/2");
        assert_eq!(show("x = 2/3 to frac\nx + 1"), "5/3");
        assert_eq!(show("0.1 + 0.2 to frac"), "3/10");
        assert_eq!(show("2 ** -2"), "0.25");
        assert_eq!(show("1/3 + 0.5"), "0.833333");
        assert_eq!(show("[1/2 == 0.5, 1/3 < 1/2, round(7/2), floor(-7/2), int(-7/2)]"), "[true, true, 4, -4, -3]");
        assert_eq!(show("str(1/3)"), "0.3333333333333333");
        assert_eq!(show("(7/2 to frac) % 1"), "1/2");
        assert_eq!(
            show(
                "h = 1/2
h km to m"
            ),
            "500 m"
        );
    }

    fn type_of(src: &str) -> &'static str {
        match eval(src) {
            Value::Int(..) => "Int",
            Value::Big(..) => "Big",
            _ => "other",
        }
    }

    #[test]
    fn records_and_mutation() {
        let src = r#"
u = {name: "ann", tags: []}
u.age = 30
u.tags.push("x")
u.tags[0] = "y"
[u.age, u.tags, u.missing]"#;
        assert_eq!(show(src), r#"[30, ["y"], nil]"#);
    }
}
