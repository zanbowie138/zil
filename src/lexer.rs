use crate::Error;
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")]
#[logos(skip(r"#[^\n]*", allow_greedy = true))]
pub enum Tok {
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
    #[token("to")]
    To,
    #[token("return")]
    Return,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("nil")]
    Nil,

    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9]+)?", |l| l.slice().replace('_', "").parse().ok())]
    #[regex(r"[0-9][0-9_]*[eE][+-]?[0-9]+", |l| l.slice().replace('_', "").parse().ok())]
    Float(f64),
    #[regex(r"[0-9][0-9_]*", |l| l.slice().replace('_', "").parse().ok())]
    Int(i64),
    /// Integer literal that keeps its base: `0xff`, `0b101`, `0o17`, `36#z`.
    #[regex(r"0x[0-9a-fA-F_]+", |l| radix(&l.slice()[2..], 16))]
    #[regex(r"0b[01_]+", |l| radix(&l.slice()[2..], 2))]
    #[regex(r"0o[0-7_]+", |l| radix(&l.slice()[2..], 8))]
    #[regex(r"[0-9]+#[0-9a-zA-Z_]+", |l| based(l.slice()))]
    Based((i64, u32)),
    /// Raw contents between the quotes; escapes and `{}` interpolation are handled by the parser.
    #[token("\"", lex_string)]
    Str(String),
    #[regex(r#"r"[^"]*""#, |l| { let s = l.slice(); s[2..s.len() - 1].to_string() })]
    Regex(String),
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
    #[token("..")]
    DotDot,
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
    #[token("**")]
    StarStar,
    #[token("/")]
    Slash,
    #[token("//")]
    SlashSlash,
    #[token("%")]
    Percent,
    #[token("^")]
    Caret,
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
    #[token("&")]
    Amp,
    #[token("|")]
    Bar,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token("~")]
    Tilde,

    Eof,
}

fn lex_string(l: &mut logos::Lexer<Tok>) -> Option<String> {
    let rest = l.remainder();
    let end = string_end(rest)?;
    l.bump(end + 1);
    Some(rest[..end].to_string())
}

/// Index of the closing quote, skipping escapes and quotes nested inside `{...}` interpolations.
fn string_end(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let (mut i, mut depth) = (0, 0);
    while i < b.len() {
        match b[i] {
            b'\\' => i += 1,
            b'"' if depth == 0 => return Some(i),
            b'"' => i += 1 + string_end(&s[i + 1..])?,
            b'{' => depth += 1,
            b'}' if depth > 0 => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    None
}

/// `36#z`
fn based(s: &str) -> Option<(i64, u32)> {
    let (b, d) = s.split_once('#')?;
    radix(d, b.parse().ok()?)
}

fn radix(digits: &str, base: u32) -> Option<(i64, u32)> {
    if !(2..=36).contains(&base) {
        return None;
    }
    Some((i64::from_str_radix(&digits.replace('_', ""), base).ok()?, base))
}

pub type Span = std::ops::Range<usize>;

/// Lex `src`, shifting spans by `offset` (used for `{...}` inside strings).
pub fn lex_at(src: &str, offset: usize) -> Result<Vec<(Tok, Span)>, Error> {
    let mut toks = Vec::new();
    for (tok, span) in Tok::lexer(src).spanned() {
        let span = span.start + offset..span.end + offset;
        match tok {
            Ok(t) => toks.push((t, span)),
            Err(()) => return Err(Error::new("unexpected character", span)),
        }
    }
    let end = src.len() + offset;
    toks.push((Tok::Eof, end..end));
    Ok(toks)
}

pub fn lex(src: &str) -> Result<Vec<(Tok, Span)>, Error> {
    lex_at(src, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(src: &str) -> Vec<Tok> {
        lex(src).unwrap().into_iter().map(|(t, _)| t).collect()
    }

    #[test]
    fn numbers() {
        assert_eq!(
            toks("0xff 0b101 36#z 1_000 2.5e3 1..3 # c"),
            vec![
                Tok::Based((255, 16)),
                Tok::Based((5, 2)),
                Tok::Based((35, 36)),
                Tok::Int(1000),
                Tok::Float(2500.0),
                Tok::Int(1),
                Tok::DotDot,
                Tok::Int(3),
                Tok::Eof
            ]
        );
    }

    #[test]
    fn units_and_strings() {
        assert_eq!(
            toks(r#"5km to mi "a{f("}")}" r"\d+" # c"#),
            vec![Tok::Int(5), Tok::Ident("km".into()), Tok::To, Tok::Ident("mi".into()), Tok::Str(r#"a{f("}")}"#.into()), Tok::Regex(r"\d+".into()), Tok::Eof]
        );
    }
}
