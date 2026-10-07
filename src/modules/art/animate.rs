//! `animate`: frames redrawn in place in the terminal.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Fail};
use crate::value::{Value, num};
use std::io::{IsTerminal, Write};

pub fn animate(it: &mut Interp, args: &[Value], span: &Span) -> Call {
    use Value::*;
    let (frames, rest) = match args {
        [f @ (List(_) | Fn(_) | Builtin(..)), rest @ ..] if rest.len() <= 2 => (f, rest),
        _ => return Err(Fail::BadArgs),
    };
    let fps = match rest.first() {
        None => 10.0,
        Some(v) => num(v).filter(|f| (0.1..=120.0).contains(f)).ok_or(Fail::Arg(1, "fps must be a number from 0.1 to 120".into()))?,
    };
    let count = match rest.get(1) {
        None => None,
        Some(Int(n, _)) if (1..=100_000).contains(n) => Some(*n as usize),
        Some(_) => return Err(Fail::Arg(2, "count must be an int from 1 to 100000".into())),
    };
    let list = if let List(l) = frames { Some(l.borrow().clone()) } else { None };
    let total = match &list {
        Some(l) if l.is_empty() => return Err(Fail::Arg(0, "no frames".into())),
        Some(l) => l.len() * count.unwrap_or(1),
        None => count.unwrap_or(60),
    };
    let tty = std::io::stdout().is_terminal();
    let mut out = std::io::stdout();
    let mut shown = 0;
    // ponytail: Ctrl-C ends the whole process mid-animation; add a SIGINT handler if that hurts in the REPL.
    for i in 0..total {
        let frame = match &list {
            Some(l) => l[i % l.len()].clone(),
            None => it.call(frames, vec![Value::int(i as i64)], span)?,
        };
        if !tty && i + 1 < total {
            continue;
        }
        let text = frame.to_string();
        if shown > 0 {
            // Back up over the previous frame and clear from there down.
            _ = write!(out, "\x1b[{shown}F\x1b[J");
        }
        _ = writeln!(out, "{text}");
        _ = out.flush();
        shown = text.lines().count().max(1);
        if tty && i + 1 < total {
            std::thread::sleep(std::time::Duration::from_secs_f64(1.0 / fps));
        }
    }
    Ok(Nil)
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn animate() {
        // Tests don't run on a terminal, so only the last frame prints and nothing sleeps.
        assert_eq!(show(r#"animate(["a", "b"], 1, 3)"#), "nil");
        assert_eq!(show(r#"animate(|i| str(i), 1, 5)"#), "nil");
        assert!(try_eval("animate([])").is_err());
        assert!(try_eval(r#"animate(["a"], 0)"#).is_err());
        assert!(try_eval(r#"animate(|i| 1 / 0, 5, 2)"#).is_err());
    }
}
