use crate::interp::{Interp, Value};
use crate::lexer::Span;
use crate::Error;
use indexmap::IndexMap;
use std::io::Read;

pub const NAMES: &[&str] = &[
    "print", "type", "len", "str", "int", "float", "range", "push", "map", "filter", "reduce", "keys",
    "values", "join", "split", "read_file", "write_file", "read_stdin", "parse_json", "to_json",
    "parse_csv", "to_csv",
];

pub fn call(it: &mut Interp, name: &str, args: Vec<Value>, span: &Span) -> Result<Value, Error> {
    let err = |msg: String| Error::new(format!("{name}: {msg}"), span.clone());
    let bad_args = || {
        let types: Vec<_> = args.iter().map(Value::type_name).collect();
        err(format!("unsupported arguments ({})", types.join(", ")))
    };
    use Value::*;
    Ok(match (name, args.as_slice()) {
        ("print", vs) => {
            let parts: Vec<String> = vs.iter().map(Value::to_string).collect();
            println!("{}", parts.join(" "));
            Nil
        }
        ("type", [v]) => Value::str(v.type_name()),
        ("len", [Str(s)]) => Int(s.chars().count() as i64),
        ("len", [List(l)]) => Int(l.borrow().len() as i64),
        ("len", [Map(m)]) => Int(m.borrow().len() as i64),
        ("str", [v]) => Value::str(v.to_string()),
        ("int", [Int(n)]) => Int(*n),
        ("int", [Float(n)]) => Int(*n as i64),
        ("int", [Str(s)]) => Int(s.trim().parse().map_err(|_| err(format!("cannot parse {s:?}")))?),
        ("float", [Int(n)]) => Float(*n as f64),
        ("float", [Float(n)]) => Float(*n),
        ("float", [Str(s)]) => Float(s.trim().parse().map_err(|_| err(format!("cannot parse {s:?}")))?),
        ("range", [Int(n)]) => Value::list((0..*n).map(Int).collect()),
        ("range", [Int(a), Int(b)]) => Value::list((*a..*b).map(Int).collect()),
        ("push", [List(l), v]) => {
            l.borrow_mut().push(v.clone());
            List(l.clone())
        }
        ("map", [List(l), f]) => {
            let items = l.borrow().clone();
            let mut out = Vec::with_capacity(items.len());
            for v in items {
                out.push(it.call(f, vec![v], span)?);
            }
            Value::list(out)
        }
        ("filter", [List(l), f]) => {
            let items = l.borrow().clone();
            let mut out = Vec::new();
            for v in items {
                if it.call(f, vec![v.clone()], span)?.truthy() {
                    out.push(v);
                }
            }
            Value::list(out)
        }
        ("reduce", [List(l), init, f]) => {
            let items = l.borrow().clone();
            let mut acc = init.clone();
            for v in items {
                acc = it.call(f, vec![acc, v], span)?;
            }
            acc
        }
        ("keys", [Map(m)]) => Value::list(m.borrow().keys().map(|k| Value::str(k.as_str())).collect()),
        ("values", [Map(m)]) => Value::list(m.borrow().values().cloned().collect()),
        ("join", [List(l), Str(sep)]) => {
            let parts: Vec<String> = l.borrow().iter().map(Value::to_string).collect();
            Value::str(parts.join(sep))
        }
        ("split", [Str(s), Str(sep)]) => Value::list(s.split(&**sep).map(Value::str).collect()),
        ("read_file", [Str(path)]) => Value::str(std::fs::read_to_string(&**path).map_err(|e| err(e.to_string()))?),
        ("write_file", [Str(path), Str(data)]) => {
            std::fs::write(&**path, &**data).map_err(|e| err(e.to_string()))?;
            Nil
        }
        ("read_stdin", []) => {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).map_err(|e| err(e.to_string()))?;
            Value::str(s)
        }
        ("parse_json", [Str(s)]) => from_json(serde_json::from_str(s).map_err(|e| err(e.to_string()))?),
        ("to_json", [v]) => Value::str(to_json(v).map_err(err)?.to_string()),
        ("to_json", [v, Bool(true)]) => Value::str(serde_json::to_string_pretty(&to_json(v).map_err(err)?).unwrap()),
        ("parse_csv", [Str(s)]) => parse_csv(s).map_err(|e| err(e.to_string()))?,
        ("to_csv", [List(rows)]) => Value::str(to_csv(&rows.borrow()).map_err(err)?),
        _ => return Err(bad_args()),
    })
}

fn from_json(j: serde_json::Value) -> Value {
    use serde_json::Value as J;
    match j {
        J::Null => Value::Nil,
        J::Bool(b) => Value::Bool(b),
        J::Number(n) => n.as_i64().map(Value::Int).unwrap_or_else(|| Value::Float(n.as_f64().unwrap_or(f64::NAN))),
        J::String(s) => Value::str(s),
        J::Array(a) => Value::list(a.into_iter().map(from_json).collect()),
        J::Object(o) => Value::map(o.into_iter().map(|(k, v)| (k, from_json(v))).collect()),
    }
}

fn to_json(v: &Value) -> Result<serde_json::Value, String> {
    use serde_json::Value as J;
    Ok(match v {
        Value::Nil => J::Null,
        Value::Bool(b) => J::Bool(*b),
        Value::Int(n) => J::from(*n),
        Value::Float(n) => serde_json::Number::from_f64(*n).map(J::Number).unwrap_or(J::Null),
        Value::Str(s) => J::String(s.to_string()),
        Value::List(l) => J::Array(l.borrow().iter().map(to_json).collect::<Result<_, _>>()?),
        Value::Map(m) => J::Object(
            m.borrow().iter().map(|(k, v)| Ok((k.clone(), to_json(v)?))).collect::<Result<_, String>>()?,
        ),
        Value::Fn(_) | Value::Builtin(_) => return Err("cannot serialize fn".into()),
    })
}

/// Header row becomes map keys; all cells stay strings (convert with `int`/`float`).
fn parse_csv(s: &str) -> Result<Value, csv::Error> {
    let mut r = csv::Reader::from_reader(s.as_bytes());
    let headers = r.headers()?.clone();
    let mut rows = Vec::new();
    for rec in r.records() {
        let rec = rec?;
        let m: IndexMap<_, _> = headers.iter().zip(rec.iter()).map(|(h, c)| (h.to_string(), Value::str(c))).collect();
        rows.push(Value::map(m));
    }
    Ok(Value::list(rows))
}

/// Columns come from the first row's keys.
fn to_csv(rows: &[Value]) -> Result<String, String> {
    let mut w = csv::Writer::from_writer(Vec::new());
    let mut headers: Option<Vec<String>> = None;
    for row in rows {
        let Value::Map(m) = row else { return Err(format!("rows must be maps, got {}", row.type_name())) };
        let m = m.borrow();
        if headers.is_none() {
            let hs: Vec<String> = m.keys().cloned().collect();
            w.write_record(&hs).map_err(|e| e.to_string())?;
            headers = Some(hs);
        }
        let hs = headers.as_ref().unwrap();
        let cells = hs.iter().map(|h| match m.get(h) {
            None | Some(Value::Nil) => String::new(),
            Some(v) => v.to_string(),
        });
        w.write_record(cells).map_err(|e| e.to_string())?;
    }
    String::from_utf8(w.into_inner().map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::eval;

    #[test]
    fn json_roundtrip() {
        let v = eval(r#"parse_json("{\"a\": [1, 2.5, null], \"b\": true}") |> to_json"#);
        assert_eq!(v.to_string(), r#"{"a":[1,2.5,null],"b":true}"#);
    }

    #[test]
    fn csv_roundtrip() {
        let v = eval(r#"parse_csv("name,age\nann,30\nbob,25\n").map(\r -> r.name) |> join(",")"#);
        assert_eq!(v.to_string(), "ann,bob");
        let v = eval(r#"to_csv([{a: 1, b: "x"}, {a: 2}])"#);
        assert_eq!(v.to_string(), "a,b\n1,x\n2,\n");
    }

    #[test]
    fn reduce_sums() {
        assert_eq!(eval(r"range(5).reduce(0, \acc, x -> acc + x)").to_string(), "10");
    }
}
