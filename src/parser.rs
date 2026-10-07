use crate::Error;
use crate::ast::{BinOp, Expr, ExprKind, FnDef, Target, UnOp, UnitSpec};
use crate::lexer::{Span, Tok, lex_at};
use std::rc::Rc;

type PResult<T> = Result<T, Error>;

pub fn parse(toks: Vec<(Tok, Span)>) -> PResult<Vec<Expr>> {
    let mut p = Parser { toks, pos: 0, prev_end: 0, holes: 0 };
    let mut stmts = Vec::new();
    p.skip_nl();
    while *p.peek() != Tok::Eof {
        stmts.push(p.expr(0)?);
        if *p.peek() != Tok::Eof {
            p.expect(Tok::Newline, "newline")?;
        }
        p.skip_nl();
    }
    Ok(stmts)
}

struct Parser {
    toks: Vec<(Tok, Span)>,
    pos: usize,
    prev_end: usize,
    /// `_` placeholders seen so far, so `x |> f(_, 2)` knows to bind `_` instead of inserting `x`.
    holes: usize,
}

/// (left, right) binding power of infix operators; higher binds tighter.
fn infix_bp(t: &Tok) -> Option<(u8, u8)> {
    Some(match t {
        Tok::Assign | Tok::OpAssign(_) => (2, 1),
        Tok::Pipe => (4, 5),
        Tok::Or => (6, 7),
        Tok::And => (8, 9),
        Tok::Eq | Tok::Ne => (10, 11),
        Tok::Lt | Tok::Le | Tok::Gt | Tok::Ge | Tok::In => (12, 13),
        Tok::DotDot | Tok::DotDotEq => (14, 15),
        Tok::Bar => (16, 17),
        Tok::Caret => (18, 19),
        Tok::Amp => (20, 21),
        Tok::Shl | Tok::Shr => (22, 23),
        Tok::Plus | Tok::Minus => (24, 25),
        Tok::Star | Tok::Slash | Tok::SlashSlash | Tok::Percent | Tok::Of => (26, 27),
        Tok::StarStar => (30, 29),
        _ => return None,
    })
}
const TO_BP: u8 = 3;
const PREFIX_BP: u8 = 28;
const POSTFIX_BP: u8 = 32;

fn binop(t: &Tok) -> BinOp {
    match t {
        Tok::Or => BinOp::Or,
        Tok::And => BinOp::And,
        Tok::Eq => BinOp::Eq,
        Tok::Ne => BinOp::Ne,
        Tok::Lt => BinOp::Lt,
        Tok::Le => BinOp::Le,
        Tok::Gt => BinOp::Gt,
        Tok::Ge => BinOp::Ge,
        Tok::DotDot => BinOp::Range,
        Tok::DotDotEq => BinOp::RangeIncl,
        Tok::In => BinOp::In,
        Tok::Plus => BinOp::Add,
        Tok::Minus => BinOp::Sub,
        Tok::Star | Tok::Of => BinOp::Mul,
        Tok::Slash => BinOp::Div,
        Tok::SlashSlash => BinOp::IntDiv,
        Tok::Percent => BinOp::Rem,
        Tok::StarStar => BinOp::Pow,
        Tok::Caret => BinOp::BitXor,
        Tok::Amp => BinOp::BitAnd,
        Tok::Bar => BinOp::BitOr,
        Tok::Shl => BinOp::Shl,
        Tok::Shr => BinOp::Shr,
        _ => unreachable!(),
    }
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos].0
    }

    fn peek_at(&self, n: usize) -> &Tok {
        &self.toks[(self.pos + n).min(self.toks.len() - 1)].0
    }

    fn span(&self) -> Span {
        self.toks[self.pos].1.clone()
    }

    fn bump(&mut self) -> Tok {
        let (t, s) = self.toks[self.pos].clone();
        if t != Tok::Eof {
            self.pos += 1;
        }
        self.prev_end = s.end;
        t
    }

    /// No whitespace between the previous token and the current one.
    fn touching(&self) -> bool {
        self.pos > 0 && self.toks[self.pos - 1].1.end == self.toks[self.pos].1.start
    }

    fn expect(&mut self, t: Tok, what: &str) -> PResult<()> {
        if *self.peek() == t {
            self.bump();
            Ok(())
        } else {
            Err(Error::new(format!("expected {what}, found {:?}", self.peek()), self.span()))
        }
    }

    fn ident(&mut self) -> PResult<String> {
        let span = self.span();
        match self.bump() {
            Tok::Ident(s) => Ok(s),
            t => Err(Error::new(format!("expected identifier, found {t:?}"), span)),
        }
    }

    fn skip_nl(&mut self) {
        while *self.peek() == Tok::Newline {
            self.bump();
        }
    }

    /// Token after any run of newlines, so `|>` / `.` can start a continuation line.
    fn peek_past_nl(&self) -> &Tok {
        let mut i = self.pos;
        while self.toks[i].0 == Tok::Newline {
            i += 1;
        }
        &self.toks[i].0
    }

    fn mk(&self, kind: ExprKind, start: usize) -> Expr {
        Expr { kind, span: start..self.prev_end }
    }

    fn expr(&mut self, min_bp: u8) -> PResult<Expr> {
        let start = self.span().start;
        let mut lhs = self.prefix()?;
        loop {
            if *self.peek() == Tok::Newline && matches!(self.peek_past_nl(), Tok::Pipe | Tok::Dot) {
                self.skip_nl();
            }
            let t = self.peek().clone();
            if matches!(t, Tok::LParen | Tok::LBracket | Tok::Dot) {
                if POSTFIX_BP < min_bp {
                    break;
                }
                self.bump();
                let kind = match t {
                    Tok::LParen => ExprKind::Call(Box::new(lhs), self.list(Tok::RParen)?),
                    Tok::LBracket => self.index(lhs)?,
                    _ => ExprKind::Field(Box::new(lhs), self.ident()?),
                };
                lhs = self.mk(kind, start);
                continue;
            }
            if t == Tok::To {
                if TO_BP < min_bp {
                    break;
                }
                self.bump();
                let target = self.target()?;
                lhs = self.mk(ExprKind::To(Box::new(lhs), target), start);
                continue;
            }
            let Some((lbp, rbp)) = infix_bp(&t) else {
                break;
            };
            if lbp < min_bp {
                break;
            }
            self.bump();
            self.skip_nl();
            let outer_holes = (t == Tok::Pipe).then(|| std::mem::take(&mut self.holes));
            let rhs = self.expr(rbp)?;
            let hole = outer_holes.is_some_and(|outer| std::mem::replace(&mut self.holes, outer) > 0);
            let kind = match t {
                Tok::Assign | Tok::OpAssign(_) if !matches!(lhs.kind, ExprKind::Ident(_) | ExprKind::Field(..) | ExprKind::Index(..)) => {
                    return Err(Error::new("invalid assignment target", lhs.span));
                }
                Tok::Assign => ExprKind::Assign(Box::new(lhs), Box::new(rhs)),
                // ponytail: `xs[f()] += 1` evaluates `xs` and `f()` twice; desugar to a temp if side effects there ever matter.
                Tok::OpAssign(op) => {
                    let value = self.mk(arith(op, lhs.clone(), rhs), start);
                    ExprKind::Assign(Box::new(lhs), Box::new(value))
                }
                // `x |> f(_, 2)` => `(\_ -> f(_, 2))(x)`
                Tok::Pipe if hole => {
                    let span = rhs.span.clone();
                    let f = Expr { kind: ExprKind::Fn(Rc::new(FnDef { params: vec!["_".into()], body: rhs })), span };
                    ExprKind::Call(Box::new(f), vec![lhs])
                }
                // `x |> f(a)` => `f(x, a)`; `x |> f` => `f(x)`
                Tok::Pipe => match rhs.kind {
                    ExprKind::Call(f, mut args) => {
                        args.insert(0, lhs);
                        ExprKind::Call(f, args)
                    }
                    _ => ExprKind::Call(Box::new(rhs), vec![lhs]),
                },
                // `a < b < c` => Chain(a, [(<, b), (<, c)])
                Tok::Lt | Tok::Le | Tok::Gt | Tok::Ge => match lhs.kind {
                    ExprKind::Binary(op @ (BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge), a, b) => ExprKind::Chain(a, vec![(op, *b), (binop(&t), rhs)]),
                    ExprKind::Chain(a, mut rest) => {
                        rest.push((binop(&t), rhs));
                        ExprKind::Chain(a, rest)
                    }
                    _ => ExprKind::Binary(binop(&t), Box::new(lhs), Box::new(rhs)),
                },
                _ => arith(binop(&t), lhs, rhs),
            };
            lhs = self.mk(kind, start);
        }
        Ok(lhs)
    }

    fn prefix(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        let tok_span = self.span();
        if matches!(self.peek(), Tok::Ident(s) if s == "_") {
            self.holes += 1;
        }
        let kind = match self.bump() {
            Tok::Nil => ExprKind::Nil,
            Tok::True => ExprKind::Bool(true),
            Tok::False => ExprKind::Bool(false),
            t @ (Tok::Int(_) | Tok::Based(_) | Tok::Big(_) | Tok::Float(_) | Tok::Dec(_)) => {
                let kind = match t {
                    Tok::Int(n) => ExprKind::Int(n, 10),
                    Tok::Based((n, b)) => ExprKind::Int(n, b),
                    Tok::Big((n, b)) => ExprKind::Big(Rc::new(n), b),
                    Tok::Float(n) => ExprKind::Float(n),
                    // `2.0` asks for a float; `2.5` is exact.
                    Tok::Dec(r) if r.is_integer() => ExprKind::Float(r.to_integer().to_string().parse().unwrap_or(f64::NAN)),
                    Tok::Dec(r) => ExprKind::Dec(Rc::new(r)),
                    _ => unreachable!(),
                };
                // `20%` unless an operand follows: `20 % 3` and `7%3` stay remainders.
                if *self.peek() == Tok::Percent && self.touching() && !starts_operand(self.peek_at(1)) {
                    self.bump();
                    ExprKind::Percent(Box::new(self.mk(kind, start)))
                } else if self.unit_next() {
                    let num = self.mk(kind, start);
                    let qty = ExprKind::Qty(Box::new(num), self.unit_spec(true)?);
                    // `5 ft 11 in`, `1 h 30 min`: adjacent quantities add, as one literal.
                    if matches!(self.peek(), Tok::Int(_) | Tok::Float(_) | Tok::Dec(_)) && matches!(self.peek_at(1), Tok::Ident(_) | Tok::In) {
                        let lhs = self.mk(qty, start);
                        ExprKind::Binary(BinOp::Add, Box::new(lhs), Box::new(self.prefix()?))
                    } else {
                        qty
                    }
                } else {
                    kind
                }
            }
            Tok::Str(raw) => self.string(&raw, tok_span.start + 1)?,
            Tok::Regex(src) => match regex::Regex::new(&src) {
                Ok(re) => ExprKind::Regex(Rc::new(re)),
                Err(e) => return Err(Error::new(format!("bad regex: {e}"), tok_span)),
            },
            // `a m`: a variable holding a number, given a unit.
            Tok::Ident(s) if self.unit_next() => {
                let var = self.mk(ExprKind::Ident(s), start);
                ExprKind::Qty(Box::new(var), self.unit_spec(true)?)
            }
            Tok::Ident(s) => ExprKind::Ident(s),
            Tok::LParen => {
                self.skip_nl();
                let e = self.expr(0)?;
                self.skip_nl();
                self.expect(Tok::RParen, "`)`")?;
                return Ok(e);
            }
            Tok::LBracket => ExprKind::List(self.list(Tok::RBracket)?),
            Tok::LBrace => ExprKind::Map(self.map_body()?),
            Tok::Minus => ExprKind::Unary(UnOp::Neg, Box::new(self.expr(PREFIX_BP)?)),
            Tok::Bang => ExprKind::Unary(UnOp::Not, Box::new(self.expr(PREFIX_BP)?)),
            Tok::Tilde => ExprKind::Unary(UnOp::BitNot, Box::new(self.expr(PREFIX_BP)?)),
            Tok::Backslash => {
                let params = self.params(Tok::Arrow)?;
                self.expect(Tok::Arrow, "`->`")?;
                let body = self.expr(0)?;
                ExprKind::Fn(Rc::new(FnDef { params, body }))
            }
            Tok::Fn => {
                self.expect(Tok::LParen, "`(`")?;
                let params = self.params(Tok::RParen)?;
                self.expect(Tok::RParen, "`)`")?;
                let body = self.block()?;
                ExprKind::Fn(Rc::new(FnDef { params, body }))
            }
            Tok::If => {
                let cond = self.expr(0)?;
                let then = self.block()?;
                let els = if *self.peek_past_nl() == Tok::Else {
                    self.skip_nl();
                    self.bump();
                    Some(Box::new(if *self.peek() == Tok::If { self.expr(0)? } else { self.block()? }))
                } else {
                    None
                };
                ExprKind::If(Box::new(cond), Box::new(then), els)
            }
            Tok::While => {
                let cond = self.expr(0)?;
                ExprKind::While(Box::new(cond), Box::new(self.block()?))
            }
            Tok::For => {
                let name = self.ident()?;
                self.expect(Tok::In, "`in`")?;
                let iter = self.expr(0)?;
                ExprKind::For(name, Box::new(iter), Box::new(self.block()?))
            }
            Tok::Break => ExprKind::Break,
            Tok::Continue => ExprKind::Continue,
            Tok::Return => {
                let val = match self.peek() {
                    Tok::Newline | Tok::RBrace | Tok::RParen | Tok::Eof => None,
                    _ => Some(Box::new(self.expr(0)?)),
                };
                ExprKind::Return(val)
            }
            t => return Err(Error::new(format!("unexpected {t:?}"), tok_span)),
        };
        Ok(self.mk(kind, start))
    }

    /// After a number or variable: a unit follows. `in` is inches only when no expression
    /// follows it, so `5 in to cm` is inches but `5 in xs` is membership.
    fn unit_next(&self) -> bool {
        match self.peek() {
            Tok::Ident(_) => true,
            Tok::In => !matches!(
                self.peek_at(1),
                Tok::Ident(_)
                    | Tok::Int(_)
                    | Tok::Based(_)
                    | Tok::Big(_)
                    | Tok::Float(_)
                    | Tok::Dec(_)
                    | Tok::Str(_)
                    | Tok::Regex(_)
                    | Tok::LParen
                    | Tok::LBracket
                    | Tok::LBrace
            ),
            _ => false,
        }
    }

    fn params(&mut self, close: Tok) -> PResult<Vec<String>> {
        let mut params = Vec::new();
        while *self.peek() != close {
            params.push(self.ident()?);
            if *self.peek() != Tok::Comma {
                break;
            }
            self.bump();
        }
        Ok(params)
    }

    /// After `[`: `x[i]`, `x[a..b]`, `x[a..]`, `x[..b]`.
    fn index(&mut self, lhs: Expr) -> PResult<ExprKind> {
        let bound = |p: &mut Self| -> PResult<Option<Box<Expr>>> {
            Ok(match p.peek() {
                Tok::DotDot | Tok::RBracket => None,
                _ => Some(Box::new(p.expr(15)?)),
            })
        };
        let from = bound(self)?;
        let kind = if *self.peek() == Tok::DotDot {
            self.bump();
            ExprKind::Slice(Box::new(lhs), from, bound(self)?)
        } else {
            let idx = from.ok_or_else(|| Error::new("expected index", self.span()))?;
            ExprKind::Index(Box::new(lhs), idx)
        };
        self.expect(Tok::RBracket, "`]`")?;
        Ok(kind)
    }

    /// Units after a number (`touching`: `/`, `*`, `^` only continue the unit without spaces,
    /// so `60 km/h` is a unit but `60 km / h` divides) or after `to` (spaces allowed).
    fn unit_spec(&mut self, touching: bool) -> PResult<UnitSpec> {
        let mut spec = vec![self.unit_term(touching, 1)?];
        loop {
            let sign = match self.peek() {
                Tok::Star => 1,
                Tok::Slash => -1,
                _ => break,
            };
            let unit_next = matches!(self.peek_at(1), Tok::Ident(_) | Tok::In);
            if !unit_next || (touching && !(self.touching() && self.toks[self.pos].1.end == self.toks[self.pos + 1].1.start)) {
                break;
            }
            self.bump();
            spec.push(self.unit_term(touching, sign)?);
        }
        Ok(spec)
    }

    fn unit_term(&mut self, touching: bool, sign: i8) -> PResult<(String, i8)> {
        let span = self.span();
        let name = match self.bump() {
            Tok::Ident(s) => s,
            Tok::In => "in".into(),
            t => return Err(Error::new(format!("expected unit, found {t:?}"), span)),
        };
        let mut pow = 1;
        if *self.peek() == Tok::Caret && (!touching || self.touching()) {
            self.bump();
            let neg = *self.peek() == Tok::Minus;
            if neg {
                self.bump();
            }
            let span = self.span();
            pow = match self.bump() {
                Tok::Int(n) if (1..=9).contains(&n) => n as i8,
                _ => return Err(Error::new("expected unit power 1-9", span)),
            };
            if neg {
                pow = -pow;
            }
        }
        Ok((name, pow * sign))
    }

    fn target(&mut self) -> PResult<Target> {
        match self.peek().clone() {
            Tok::Ident(s) if crate::modules::target(&s).is_some() => {
                self.bump();
                let mut arg = None;
                if *self.peek() == Tok::LParen {
                    self.bump();
                    let span = self.span();
                    arg = match self.bump() {
                        Tok::Int(n) => Some(n),
                        _ => return Err(Error::new("expected an integer", span)),
                    };
                    self.expect(Tok::RParen, "`)`")?;
                }
                Ok(Target::Named(s, arg))
            }
            Tok::Str(s) => {
                self.bump();
                Ok(Target::Str(s))
            }
            _ => {
                let mut specs = vec![self.unit_spec(false)?];
                while matches!(self.peek(), Tok::Ident(_) | Tok::In) {
                    specs.push(self.unit_spec(false)?);
                }
                Ok(if specs.len() == 1 { Target::Unit(specs.pop().unwrap()) } else { Target::Units(specs) })
            }
        }
    }

    /// `raw` is the text between the quotes, starting at byte `start` of the source.
    fn string(&self, raw: &str, start: usize) -> PResult<ExprKind> {
        let span = start - 1..start + raw.len() + 1;
        let lit_expr = |s: &mut String| Expr { kind: ExprKind::Str(std::mem::take(s).into()), span: span.clone() };
        let mut parts: Vec<Expr> = Vec::new();
        let mut lit = String::new();
        let mut chars = raw.char_indices();
        while let Some((i, c)) = chars.next() {
            match c {
                '\\' => {
                    let (_, e) = chars.next().expect("lexer guarantees escape target");
                    lit.push(match e {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '0' => '\0',
                        '\\' | '"' | '{' | '}' => e,
                        _ => {
                            let at = start + i;
                            return Err(Error::new(format!("unknown escape `\\{e}`"), at..at + 1 + e.len_utf8()));
                        }
                    });
                }
                '{' => {
                    let end = close_brace(raw, i + 1)
                        .ok_or_else(|| Error::new("unclosed `{` in string (use `\\{` for a literal brace)", start + i..start + i + 1))?;
                    let (src_end, spec) = match spec_colon(&raw[i + 1..end]) {
                        Some(c) => (i + 1 + c, Some(&raw[i + 2 + c..end])),
                        None => (end, None),
                    };
                    if let Some(spec) = spec {
                        crate::modules::math::Spec::parse(spec).map_err(|m| Error::new(m, start + i..start + end + 1))?;
                    }
                    let inner_toks = lex_at(&raw[i + 1..src_end], start + i + 1)?;
                    let mut p = Parser { toks: inner_toks, pos: 0, prev_end: 0, holes: 0 };
                    p.skip_nl();
                    let e = p.expr(0)?;
                    p.skip_nl();
                    p.expect(Tok::Eof, "`}`")?;
                    let e = match spec {
                        Some(spec) => Expr { span: e.span.clone(), kind: ExprKind::Format(Box::new(e), spec.into()) },
                        None => e,
                    };
                    if !lit.is_empty() || parts.is_empty() {
                        parts.push(lit_expr(&mut lit));
                    }
                    parts.push(e);
                    while chars.as_str().len() > raw.len() - end - 1 {
                        chars.next();
                    }
                }
                _ => lit.push(c),
            }
        }
        if parts.is_empty() {
            return Ok(ExprKind::Str(lit.into()));
        }
        if !lit.is_empty() {
            parts.push(lit_expr(&mut lit));
        }
        // Leading Str part makes `+` concatenate everything.
        let mut it = parts.into_iter();
        let mut acc = it.next().unwrap();
        for p in it {
            acc = Expr { kind: ExprKind::Binary(BinOp::Add, Box::new(acc), Box::new(p)), span: span.clone() };
        }
        Ok(acc.kind)
    }

    /// Comma-separated exprs up to `close`; newlines and a trailing comma allowed.
    fn list(&mut self, close: Tok) -> PResult<Vec<Expr>> {
        let mut items = Vec::new();
        self.skip_nl();
        while *self.peek() != close {
            items.push(self.expr(0)?);
            self.skip_nl();
            if *self.peek() != Tok::Comma {
                break;
            }
            self.bump();
            self.skip_nl();
        }
        self.expect(close, "closing delimiter")?;
        Ok(items)
    }

    /// `{a: 1, "b c": 2}`; entries separated by commas and/or newlines.
    fn map_body(&mut self) -> PResult<Vec<(String, Expr)>> {
        let mut entries = Vec::new();
        self.skip_nl();
        while *self.peek() != Tok::RBrace {
            let span = self.span();
            let key = match self.bump() {
                Tok::Ident(s) => s,
                Tok::Str(s) if !s.contains(['\\', '{']) => s,
                t => return Err(Error::new(format!("expected map key, found {t:?}"), span)),
            };
            self.expect(Tok::Colon, "`:`")?;
            self.skip_nl();
            entries.push((key, self.expr(0)?));
            if *self.peek() == Tok::Comma {
                self.bump();
            }
            self.skip_nl();
        }
        self.expect(Tok::RBrace, "`}`")?;
        Ok(entries)
    }

    fn block(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        self.expect(Tok::LBrace, "`{`")?;
        let mut stmts = Vec::new();
        self.skip_nl();
        while *self.peek() != Tok::RBrace {
            stmts.push(self.expr(0)?);
            if *self.peek() != Tok::RBrace {
                self.expect(Tok::Newline, "newline or `}`")?;
            }
            self.skip_nl();
        }
        self.bump();
        Ok(self.mk(ExprKind::Block(stmts), start))
    }
}

/// `a op b`, where `x + 20%` means `x * (1 + 20%)` and `x - 20%` means `x * (1 - 20%)`.
fn arith(op: BinOp, lhs: Expr, rhs: Expr) -> ExprKind {
    if matches!(op, BinOp::Add | BinOp::Sub) && matches!(rhs.kind, ExprKind::Percent(_)) {
        let span = rhs.span.clone();
        let one = Expr { kind: ExprKind::Int(1, 10), span: span.clone() };
        let factor = Expr { kind: ExprKind::Binary(op, Box::new(one), Box::new(rhs)), span };
        return ExprKind::Binary(BinOp::Mul, Box::new(lhs), Box::new(factor));
    }
    ExprKind::Binary(op, Box::new(lhs), Box::new(rhs))
}

fn starts_operand(t: &Tok) -> bool {
    use Tok::*;
    matches!(
        t,
        Ident(_)
            | Int(_)
            | Based(_)
            | Big(_)
            | Float(_)
            | Dec(_)
            | Str(_)
            | Regex(_)
            | LParen
            | LBracket
            | LBrace
            | Minus
            | Bang
            | Tilde
            | Backslash
            | Fn
            | If
            | True
            | False
            | Nil
    )
}

/// Byte index of a `:` starting a format spec in an interpolation (`x:.2f`), outside brackets and strings.
fn spec_colon(s: &str) -> Option<usize> {
    let (mut depth, mut in_str, mut esc) = (0, false, false);
    for (i, c) in s.char_indices() {
        if in_str {
            match (esc, c) {
                (true, _) => esc = false,
                (_, '\\') => esc = true,
                (_, '"') => in_str = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ':' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Byte index of the `}` closing an interpolation that starts at `from`, skipping nested braces and strings.
fn close_brace(s: &str, from: usize) -> Option<usize> {
    let (mut depth, mut in_str, mut esc) = (1, false, false);
    for (i, c) in s[from..].char_indices() {
        if in_str {
            match (esc, c) {
                (true, _) => esc = false,
                (_, '\\') => esc = true,
                (_, '"') => in_str = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(from + i);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::lexer::lex;

    fn sexp(src: &str) -> String {
        let ast = super::parse(lex(src).unwrap()).unwrap();
        format!("{:?}", ast[0].kind)
    }

    #[test]
    fn precedence() {
        assert!(sexp("1 + 2 * 3").starts_with("Binary(Add"));
        assert!(sexp("-2 ** 2").starts_with("Unary(Neg"));
        assert!(sexp("x = 5 km to mi").starts_with("Assign"));
    }

    #[test]
    fn units_touching_vs_spaced() {
        assert!(sexp("60 km/h").contains(r#"[("km", 1), ("h", -1)]"#));
        assert!(sexp("60 km / h").starts_with("Binary(Div"));
        assert!(sexp("a m to km").starts_with("To(Expr { kind: Qty(Expr { kind: Ident"));
        assert!(sexp("5 cm to in").contains(r#"Unit([("in", 1)])"#));
        assert!(sexp("1 kg*m/s^2").contains(r#"("s", -2)"#));
    }

    #[test]
    fn interpolation_and_slices() {
        assert!(sexp(r#""a{1 + 2}b""#).starts_with("Binary(Add"));
        assert!(sexp(r#""no \{brace\}""#).starts_with("Str(\"no {brace}\")"));
        assert!(sexp("s[-3..]").starts_with("Slice("));
    }

    #[test]
    fn pipe_desugars_to_call() {
        let s = sexp("xs\n  |> map(f)\n  |> len");
        assert!(s.starts_with("Call(Expr { kind: Ident(\"len\")"), "{s}");
    }

    #[test]
    fn rejects_bad_input() {
        for src in ["1 = 2", r#""{1""#, "r\"(\"", "5 to hex(x)"] {
            assert!(lex(src).and_then(super::parse).is_err(), "{src}");
        }
    }
}
