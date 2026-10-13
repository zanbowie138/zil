use std::fs;

fn main() {
    let mut seeds: Vec<String> = fs::read_dir("../examples").unwrap().map(|e| fs::read_to_string(e.unwrap().path()).unwrap()).collect();
    for m in zil::modules::modules() {
        seeds.extend(m.fns.iter().flat_map(|f| f.examples.iter().chain(f.shown)).map(|s| s.to_string()));
        seeds.extend(m.examples.iter().chain(m.guide).flat_map(|(_, rows)| rows.iter()).map(|(_, ex)| ex.to_string()).filter(|ex| !ex.is_empty()));
    }
    for target in ["lex", "parse", "eval"] {
        let dir = format!("corpus/{target}");
        fs::create_dir_all(&dir).unwrap();
        for (i, s) in seeds.iter().enumerate() {
            fs::write(format!("{dir}/seed{i}"), s).unwrap();
        }
    }
    println!("{} seeds", seeds.len());
}
