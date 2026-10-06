use crate::Error;
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")]
#[logos(skip(r"#[^\n]*", allow_greedy = true))]
pub enum Tok {
    #[token("let")]
    Let,
    #[token("fn")]
    Fn,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("return")]
    Return,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("nil")]
    Nil,

    #[regex(r"[0-9]+\.[0-9]+", |l| l.slice().parse().ok())]
    Float(f64),
    #[regex(r"[0-9]+", |l| l.slice().parse().ok())]
    Int(i64),
    #[regex(r#""([^"\\]|\\.)*""#, |l| unescape(l.slice()))]
    Str(String),
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |l| l.slice().to_string())]
    Ident(String),

    #[token("\n")]
    Newline,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token("\\")]
    Backslash,
    #[token("->")]
    Arrow,
    #[token("|>")]
    Pipe,
    #[token("=")]
    Assign,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("!")]
    Bang,
    #[token("==")]
    Eq,
    #[token("!=")]
    Ne,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token(">")]
    Gt,
    #[token(">=")]
    Ge,
    #[token("&&")]
    And,
    #[token("||")]
    Or,

    Eof,
}

fn unescape(s: &str) -> Option<String> {
    let mut out = String::new();
    let mut chars = s[1..s.len() - 1].chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        out.push(match chars.next()? {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            c @ ('\\' | '"') => c,
            _ => return None,
        });
    }
    Some(out)
}

pub type Span = std::ops::Range<usize>;

pub fn lex(src: &str) -> Result<Vec<(Tok, Span)>, Error> {
    let mut toks = Vec::new();
    for (tok, span) in Tok::lexer(src).spanned() {
        match tok {
            Ok(t) => toks.push((t, span)),
            Err(()) => return Err(Error::new("unexpected character", span)),
        }
    }
    toks.push((Tok::Eof, src.len()..src.len()));
    Ok(toks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_basics() {
        let toks: Vec<Tok> = lex(r#"let x = 1.5 |> f("a\n") # hi"#)
            .unwrap()
            .into_iter()
            .map(|(t, _)| t)
            .collect();
        assert_eq!(
            toks,
            vec![
                Tok::Let,
                Tok::Ident("x".into()),
                Tok::Assign,
                Tok::Float(1.5),
                Tok::Pipe,
                Tok::Ident("f".into()),
                Tok::LParen,
                Tok::Str("a\n".into()),
                Tok::RParen,
                Tok::Eof,
            ]
        );
    }
}
