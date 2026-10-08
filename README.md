# zil

[![crates.io](https://img.shields.io/crates/v/zil.svg)](https://crates.io/crates/zil)
[![docs](https://img.shields.io/badge/docs-reference-blue)](https://zanbowie138.github.io/zil/)
[![license](https://img.shields.io/badge/license-MIT-green)](LICENSE.md)

zil is a calculator and scripting language with built-in support for units, currencies, dates and time zones. Numbers are exact fractions and
arbitrary-size integers, so `0.1 + 0.2 == 0.3` is `true`.

```zil
100 km / 2 h to mph                      # → 31.0686 mph
1.8 m to ft in                           # → 5 ft 10.8661 in
date("2026-12-25 18:30") to "Asia/Tokyo" # → 2026-12-26 09:30:00 +09:00
0.1 + 0.2 == 0.3                         # → true
2 ** 100                                 # → 1267650600228229401496703205376
"a1b22c333".nums.sum                     # → 356
```

## Features

- **Units.** Quantities carry their units through arithmetic. `4.7 GB / 50 Mbps` gives a time, and adding meters to seconds is an error. `to`
  converts between compatible units.
- **Exact numbers.** Fractions stay fractions and integers grow as needed. Floats are only used when you ask for them.
- **Large standard library.** Dates, regex, hashing, statistics, calculus, colors, IP addresses and ASCII art are built in. There is nothing to
  import.
- **Error messages.** Errors point to the location in the source and often suggest a fix.
- **Built-in help.** `help(upper)` shows examples for a function. `help("km")` lists units.

## Install

Download a prebuilt binary from the [latest release](https://github.com/zanbowie138/zil/releases/latest):

| Platform              | Download                                                                                                                  |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| Windows x64           | [zil-x86_64-pc-windows-msvc.zip](https://github.com/zanbowie138/zil/releases/latest/download/zil-x86_64-pc-windows-msvc.zip) |
| macOS (Apple Silicon) | [zil-aarch64-apple-darwin.tar.gz](https://github.com/zanbowie138/zil/releases/latest/download/zil-aarch64-apple-darwin.tar.gz) |
| macOS (Intel)         | [zil-x86_64-apple-darwin.tar.gz](https://github.com/zanbowie138/zil/releases/latest/download/zil-x86_64-apple-darwin.tar.gz) |
| Linux x64             | [zil-x86_64-unknown-linux-gnu.tar.gz](https://github.com/zanbowie138/zil/releases/latest/download/zil-x86_64-unknown-linux-gnu.tar.gz) |
| Linux ARM64           | [zil-aarch64-unknown-linux-gnu.tar.gz](https://github.com/zanbowie138/zil/releases/latest/download/zil-aarch64-unknown-linux-gnu.tar.gz) |

Or build it with Cargo:

```sh
cargo install zil
```

To install the latest version from git:

```sh
cargo install --git https://github.com/zanbowie138/zil
```

## Usage

```
zil                        start the REPL
zil -e <code> [args...]    evaluate code and print the result
zil <file.zil> [args...]   run a script; args() lists the extra arguments
zil --docs <dir>           write the mdBook reference into dir
zil --version              print the version
```

The REPL keeps history, accepts multi-line input, and stores the last result in `_`. Piped input is available as `input`:

```sh
echo "hi there" | zil -e 'input.upper'   # → HI THERE
```

## Examples

### Units and currencies

```zil
350 F to C                      # → 176.667 C
4.7 GB / 50 Mbps to min         # → 12.5333 min
9.81 m/s^2 * 70 kg to N         # → 686.7 N
(5 m ± 0.2 m) * (3 m ± 0.1 m)   # → 15 ± 0.781025 m^2
allow_network_access()          # rates are fetched online, so opt in first
100 USD to EUR                  # → €88.87
```

### Dates and times

```zil
date("2026-12-25") - date("2026-10-07") to d   # → 79 d
now + 90 d                                     # → 2027-01-05 20:39:42 -06:00
date("2026-10-07").weekday                     # → Wednesday
```

### Math

```zil
1/3 + 1/6 to frac              # → 1/2
factorial(30)                  # → 265252859812191058636308480000000
is_prime(2 ** 61 - 1)          # → true
csqrt(-4)                      # → 2i
integrate(|x| x ** 2, 0, 3)    # → 9
255 to hex                     # → 0xff
```

### Scripting

zil has variables, functions, loops, maps and string interpolation. Everything is an expression.

```zil
fib = fn(n) {
  if n < 2 { return n }
  fib(n - 1) + fib(n - 2)
}
print((1..11).map(fib))        # [1, 1, 2, 3, 5, 8, 13, 21, 34, 55]

scores = {ana: 91, ben: 78}
for name in scores {
  grade = if scores[name] >= 90 { "A" } else { "B" }
  print("{name.capitalize}: {grade}")
}

"the quick brown fox".split.map(|w| w.capitalize).join(" ")   # → The Quick Brown Fox
[3, 1, 4, 1, 5, 9] |> sort |> map(|x| x * 2)                  # → [2, 2, 6, 8, 10, 18]
"hello".sha256[..12]                                          # → 2cf24dba5fb0
```

### Text art

```zil
cowsay("moo")
banner("zil", "slant")   # fonts() lists the rest
mandelbrot(40, 12)
maze(10, 4)
```

```
 _____                          ...:
< moo >                      ...:@@@..
 -----                    ..#+=@@@@@@@:#:.
   \              .........#@@@@@@@@@@@@-.
    \            ..:@@@@@:@@@@@@@@@@@@@@@-
         (__)  @@@@@@@@@@@@@@@@@@@@@@@@@@-..
         (oo)    ..:@@@@@:@@@@@@@@@@@@@@@-
  /-------\/      .........#@@@@@@@@@@@@-.
 / |     ||               ..#+=@@@@@@@:#:.
*  ||----||                  ...:@@@..
   ~~    ~~                     ...:
```

`examples/demo.zil` has more examples. Type `help` in the REPL to browse the builtins.

## Documentation

The reference at [zanbowie138.github.io/zil](https://zanbowie138.github.io/zil/) is generated from the builtins on every push. The language syntax
is described in [`docs/syntax.md`](docs/syntax.md). To build the reference locally:

```sh
cargo run -- --docs docs/book/src && mdbook serve docs/book
```

## Building

```sh
git clone https://github.com/zanbowie138/zil
cd zil
cargo build --release   # binary at target/release/zil
cargo test
```

## Data sources

Country and city data in `src/modules/geo/` comes from [GeoNames](https://www.geonames.org/) under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/),
trimmed to capitals and cities with more than 500,000 people.

Currency rates are provided by [Exchange Rate API](https://www.exchangerate-api.com).

## License

[MIT](LICENSE.md)
