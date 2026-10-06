use crate::ast::{BinOp, Expr, ExprKind, FnDef, UnOp};
use crate::lexer::{Span, Tok};
use crate::Error;
use std::rc::Rc;

type PResult<T> = Result<T, Error>;

pub fn parse(toks: Vec<(Tok, Span)>) -> PResult<Vec<Expr>> {
    let mut p = Parser { toks, pos: 0, prev_end: 0 };
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
}

/// (left, right) binding power of infix operators; higher binds tighter.
fn infix_bp(t: &Tok) -> Option<(u8, u8)> {
    Some(match t {
        Tok::Assign => (2, 1),
        Tok::Pipe => (3, 4),
        Tok::Or => (5, 6),
        Tok::And => (7, 8),
        Tok::Eq | Tok::Ne => (9, 10),
        Tok::Lt | Tok::Le | Tok::Gt | Tok::Ge => (11, 12),
        Tok::Plus | Tok::Minus => (13, 14),
        Tok::Star | Tok::Slash | Tok::Percent => (15, 16),
        _ => return None,
    })
}
const PREFIX_BP: u8 = 17;
const POSTFIX_BP: u8 = 19;

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
        Tok::Plus => BinOp::Add,
        Tok::Minus => BinOp::Sub,
        Tok::Star => BinOp::Mul,
        Tok::Slash => BinOp::Div,
        Tok::Percent => BinOp::Rem,
        _ => unreachable!(),
    }
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos].0
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

    fn expect(&mut self, t: Tok, what: &str) -> PResult<()> {
        if *self.peek() == t {
            self.bump();
            Ok(())
        } else {
            Err(Error::new(format!("expected {what}, found {:?}", self.peek()), self.span()))
        }
    }

    fn ident(&mut self) -> PResult<String> {
        match self.bump() {
            Tok::Ident(s) => Ok(s),
            t => Err(Error::new(format!("expected identifier, found {t:?}"), self.toks[self.pos - 1].1.clone())),
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
                    Tok::LBracket => {
                        self.skip_nl();
                        let idx = self.expr(0)?;
                        self.skip_nl();
                        self.expect(Tok::RBracket, "`]`")?;
                        ExprKind::Index(Box::new(lhs), Box::new(idx))
                    }
                    _ => ExprKind::Field(Box::new(lhs), self.ident()?),
                };
                lhs = self.mk(kind, start);
                continue;
            }
            let Some((lbp, rbp)) = infix_bp(&t) else { break };
            if lbp < min_bp {
                break;
            }
            self.bump();
            self.skip_nl();
            let rhs = self.expr(rbp)?;
            let kind = match t {
                Tok::Assign => {
                    if !matches!(lhs.kind, ExprKind::Ident(_) | ExprKind::Field(..) | ExprKind::Index(..)) {
                        return Err(Error::new("invalid assignment target", lhs.span));
                    }
                    ExprKind::Assign(Box::new(lhs), Box::new(rhs))
                }
                // `x |> f(a)` => `f(x, a)`; `x |> f` => `f(x)`
                Tok::Pipe => match rhs.kind {
                    ExprKind::Call(f, mut args) => {
                        args.insert(0, lhs);
                        ExprKind::Call(f, args)
                    }
                    _ => ExprKind::Call(Box::new(rhs), vec![lhs]),
                },
                _ => ExprKind::Binary(binop(&t), Box::new(lhs), Box::new(rhs)),
            };
            lhs = self.mk(kind, start);
        }
        Ok(lhs)
    }

    fn prefix(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        let tok_span = self.span();
        let kind = match self.bump() {
            Tok::Nil => ExprKind::Nil,
            Tok::True => ExprKind::Bool(true),
            Tok::False => ExprKind::Bool(false),
            Tok::Int(n) => ExprKind::Int(n),
            Tok::Float(n) => ExprKind::Float(n),
            Tok::Str(s) => ExprKind::Str(s.into()),
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
            Tok::Backslash => {
                let mut params = Vec::new();
                while *self.peek() != Tok::Arrow {
                    params.push(self.ident()?);
                    if *self.peek() != Tok::Comma {
                        break;
                    }
                    self.bump();
                }
                self.expect(Tok::Arrow, "`->`")?;
                let body = self.expr(0)?;
                ExprKind::Fn(Rc::new(FnDef { params, body }))
            }
            Tok::Fn => {
                self.expect(Tok::LParen, "`(`")?;
                let mut params = Vec::new();
                while *self.peek() != Tok::RParen {
                    params.push(self.ident()?);
                    if *self.peek() != Tok::Comma {
                        break;
                    }
                    self.bump();
                }
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
            Tok::Let => {
                let name = self.ident()?;
                self.expect(Tok::Assign, "`=`")?;
                self.skip_nl();
                ExprKind::Let(name, Box::new(self.expr(0)?))
            }
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
            let key = match self.bump() {
                Tok::Ident(s) | Tok::Str(s) => s,
                t => return Err(Error::new(format!("expected map key, found {t:?}"), self.toks[self.pos - 1].1.clone())),
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

#[cfg(test)]
mod tests {
    use crate::lexer::lex;

    fn sexp(src: &str) -> String {
        let ast = super::parse(lex(src).unwrap()).unwrap();
        format!("{:?}", ast[0].kind)
    }

    #[test]
    fn precedence() {
        let s = sexp("1 + 2 * 3");
        assert!(s.starts_with("Binary(Add"), "{s}");
    }

    #[test]
    fn pipe_desugars_to_call() {
        let s = sexp("xs\n  |> map(f)\n  |> len");
        assert!(s.starts_with("Call(Expr { kind: Ident(\"len\")"), "{s}");
        assert!(s.contains("Ident(\"map\")"), "{s}");
    }

    #[test]
    fn rejects_bad_assign() {
        assert!(super::parse(lex("1 = 2").unwrap()).is_err());
    }
}
