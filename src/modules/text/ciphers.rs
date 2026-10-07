//! Classic ciphers and codes: Caesar, ROT13, Atbash, Vigenère, Morse, Braille. For fun, not secrecy.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;

pub const MODULE: Module = Module {
    name: "ciphers",
    about: "Caesar, ROT13, Atbash, Vigenère, Morse, Braille (for fun, not secrecy)",
    #[rustfmt::skip]
    examples: &[
        ("ciphers", &[
            ("SOS", r#""sos".morse"#),
            ("and back", r#""... --- ...".morse"#),
            ("spoilers", r#""the butler did it".rot13"#),
            ("crack a Caesar", r#"(1..26).map(|k| "Wkh hdjoh odqgv".caesar(-k)).grep("eagle")"#),
            ("Vigenère round trip", r#""attack at dawn".vigenere("lemon").unvigenere("lemon")"#),
            ("Braille", r#""hello".braille"#),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("caesar", "caesar(s, shift)", "shift each letter by shift places; negative shifts decode", &[r#""hello".caesar(3)"#, r#""khoor".caesar(-3)"#], &["rot13", "vigenere"]),
    doc("rot13", "rot13(s)", "Caesar by 13; applying it twice gives the original", &[r#""hello".rot13"#], &["caesar"]),
    doc("atbash", "atbash(s)", "mirror the alphabet: a <-> z, b <-> y", &[r#""wizard".atbash"#], &["caesar"]),
    doc("vigenere", "vigenere(s, key)", "Caesar with a shift per letter from a key word", &[r#""attack at dawn".vigenere("lemon")"#], &["unvigenere", "caesar"]),
    doc("unvigenere", "unvigenere(s, key)", "undo vigenere", &[r#""lxfopv ef rnhr".unvigenere("lemon")"#], &["vigenere"]),
    doc("morse", "morse(s)", "text to Morse code, or Morse (dots, dashes, / between words) back to text", &[r#""sos".morse"#, r#"".... .. / - .... . .-. .".morse"#], &["braille"]),
    doc("braille", "braille(s)", "text to Grade 1 Braille cells, or Braille back to text", &[r#""hello world".braille"#, r#""⠓⠊".braille"#], &["morse"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(Value::str(match (name, args) {
        ("caesar", [Str(s), Int(k, _)]) => shift(s, |_| *k),
        ("rot13", [Str(s)]) => shift(s, |_| 13),
        ("atbash", [Str(s)]) => map_letters(s, |i| 25 - i),
        ("vigenere" | "unvigenere", [Str(s), Str(key)]) => {
            let key: Vec<i64> = key.chars().filter(char::is_ascii_alphabetic).map(|c| (c.to_ascii_lowercase() as u8 - b'a') as i64).collect();
            if key.is_empty() {
                return Err("key needs at least one letter".into());
            }
            let sign = if name == "vigenere" { 1 } else { -1 };
            shift(s, |n| sign * key[n % key.len()])
        }
        ("morse", [Str(s)]) if s.chars().all(|c| ".-/ ".contains(c)) => {
            let word = |w: &str| w.split_whitespace().map(|code| MORSE.iter().find(|m| m.1 == code).map_or('?', |m| m.0)).collect::<String>();
            s.split('/').map(word).collect::<Vec<_>>().join(" ")
        }
        ("morse", [Str(s)]) => {
            let word = |w: &str| w.chars().filter_map(|c| MORSE.iter().find(|m| m.0 == c.to_ascii_uppercase()).map(|m| m.1)).collect::<Vec<_>>().join(" ");
            s.split_whitespace().map(word).collect::<Vec<_>>().join(" / ")
        }
        ("braille", [Str(s)]) => s
            .chars()
            .map(|c| match BRAILLE.chars().position(|b| b == c) {
                Some(i) => LETTERS.as_bytes()[i] as char,
                None => LETTERS.find(c.to_ascii_lowercase()).map_or(c, |i| BRAILLE.chars().nth(i).unwrap()),
            })
            .collect(),
        _ => return Err(Fail::BadArgs),
    }))
}

/// Shifts the n-th letter by `by(n)`, keeping case; other characters pass through.
fn shift(s: &str, by: impl Fn(usize) -> i64) -> String {
    let mut n = 0;
    map_letters(s, |i| {
        let k = by(n);
        n += 1;
        (i as i64 + k).rem_euclid(26) as u8
    })
}

fn map_letters(s: &str, mut f: impl FnMut(u8) -> u8) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' => (b'a' + f(c as u8 - b'a')) as char,
            'A'..='Z' => (b'A' + f(c as u8 - b'A')) as char,
            _ => c,
        })
        .collect()
}

const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz ";
const BRAILLE: &str = "⠁⠃⠉⠙⠑⠋⠛⠓⠊⠚⠅⠇⠍⠝⠕⠏⠟⠗⠎⠞⠥⠧⠺⠭⠽⠵⠀";

#[rustfmt::skip]
const MORSE: &[(char, &str)] = &[
    ('A', ".-"), ('B', "-..."), ('C', "-.-."), ('D', "-.."), ('E', "."), ('F', "..-."), ('G', "--."), ('H', "...."),
    ('I', ".."), ('J', ".---"), ('K', "-.-"), ('L', ".-.."), ('M', "--"), ('N', "-."), ('O', "---"), ('P', ".--."),
    ('Q', "--.-"), ('R', ".-."), ('S', "..."), ('T', "-"), ('U', "..-"), ('V', "...-"), ('W', ".--"), ('X', "-..-"),
    ('Y', "-.--"), ('Z', "--.."),
    ('0', "-----"), ('1', ".----"), ('2', "..---"), ('3', "...--"), ('4', "....-"), ('5', "....."), ('6', "-...."),
    ('7', "--..."), ('8', "---.."), ('9', "----."),
    ('.', ".-.-.-"), (',', "--..--"), ('?', "..--.."), ('!', "-.-.--"), ('\'', ".----."), ('/', "-..-."), ('(', "-.--."),
    (')', "-.--.-"), ('&', ".-..."), (':', "---..."), ('=', "-...-"), ('+', ".-.-."), ('-', "-....-"), ('"', ".-..-."), ('@', ".--.-."),
];

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn ciphers() {
        assert_eq!(show(r#""Hello, World!".caesar(3)"#), "Khoor, Zruog!");
        assert_eq!(show(r#""abc".caesar(-1)"#), "zab");
        assert_eq!(show(r#""Hello".rot13.rot13"#), "Hello");
        assert_eq!(show(r#""wizard".atbash"#), "draziw");
        assert_eq!(show(r#""ATTACK AT DAWN".vigenere("LEMON")"#), "LXFOPV EF RNHR");
        assert_eq!(show(r#""lxfopv ef rnhr".unvigenere("lemon")"#), "attack at dawn");
        assert_eq!(show(r#""SOS hi".morse"#), "... --- ... / .... ..");
        assert_eq!(show(r#""... --- ... / .... ..".morse"#), "SOS HI");
        assert_eq!(show(r#""hi there".braille.braille"#), "hi there");
        assert!(try_eval(r#""x".vigenere("123")"#).is_err());
    }
}
