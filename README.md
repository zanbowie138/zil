# zil

An expression calculator and scripting language with units, dates, exact fractions and big ints.

```zil
100 km / 2 h to mph                      # → 31.0686 mph
1.8 m to ft in                           # → "5 ft 10.8661 in"
date("2026-12-25 18:30") to "Asia/Tokyo" # → 2026-12-26 09:30:00 +09:00
0.1 + 0.2 == 0.3                         # → true
2 ** 100                                 # → 1267650600228229401496703205376
"a1b22c333".nums.sum                     # → 356
```

## Install

```sh
cargo install --git https://github.com/zanbowie138/zil
```

## Usage

```
zil                        start the REPL
zil -e <code> [args...]   evaluate code and print the result
zil <file.zil> [args...]  run a script; args() lists the extra arguments
zil --docs <dir>          write the mdBook reference into dir
```

Try `zil examples/demo.zil`, or `help` inside the REPL.

## Docs

The full reference lives in [`docs/book`](docs/book/src/index.md) and is generated from the builtins themselves:

```sh
cargo run -- --docs docs/book/src
```
