# zil roadmap

Planned features in four work sessions. Each session groups features that share code, so it can be built and tested on its own. Details get settled when a feature is built.

## Session 1: numbers

All in `math`/`value.rs`. Formatting comes first because sprintf-style `format` reuses its spec parser.

### Number formatting + sprintf-style format
- `.fixed(2)`, `.sci`, `.percent`, `.commas` (`1234567.commas` → `"1,234,567"`). On quantities they keep the unit: `(5 km to mi).fixed(1)` → `"3.1 mi"`.
- Format specs in interpolation: `"{x:.2f}"`, `"{n:>8}"`, `"{n:08x}"`, `"{x:,}"`; also `format("%5.2f", x)`. One spec parser serves both.

### Stats
- `median`, `mode`, `stdev`, `variance`, `percentile(p)`, `product`. Unit-aware like `sum`: `[1 m, 3 m].median` → `2 m`.

### Number theory
- `gcd`, `lcm`, `is_prime`, `factors`, `factorial`, `choose(n, k)`, `mod_pow(b, e, m)`. Exact via big ints.

### More math
- `atan2`, `hypot`, `exp`, `cbrt`, `sinh` `cosh` `tanh`, `clamp`, `sign`, `trunc`, `round(x, to: 0.25)`.
- Constants `tau`, `phi`, `inf`, `nan`, plus `is_nan`.

### Bit helpers
- `popcount`, `bit(n, i)`, `set_bit`, `clear_bit`, `rotl`/`rotr(n, k, width)`, `byteswap(n, width)`, `to bits` (`0b1111_0000`).

### Percent literals
- `20%` is `0.2`; `50 + 10%` → `55`; `20% of 50` → `10` (`of` becomes a keyword).

### Complex numbers
- `3 + 4i` literals; `abs`, `arg`, `conj`, `re`, `im`. `sqrt(-1)` stays an error unless an operand is already complex.
- The largest item here, since it adds a `Value` variant that arithmetic must handle. Drop it if the session runs long.

## Session 2: strings & lists

Pure builtins with no new syntax, so it's quick and low risk.

### Case conversion
- `title`, `snake`, `camel`, `pascal`, `kebab`, `swapcase`, `slug`. Words split on case changes, `_`, `-` and spaces.

### Text utilities
- `pad_left`, `pad_right`, `center`, `truncate(n)`, `wrap(width)`, `dedent`, `indent(n)`.
- `levenshtein`, `similarity`, `word_count`, `strip_ansi`, `escape_html`/`unescape_html`.

### More hashes and encodings
- `sha1`, `sha512`, `crc32`, `base32`, `rot13`, `encode("html")`, `jwt_decode` (header + payload, no verification).

### Missing list/map basics
- Lists: `zip`, `enumerate`, `flatten`, `take`/`drop`, `chunk(n)`, `window(n)`, `group_by(f)`, `count_by(f)`/`tally`, `partition(f)`, `any`/`all`, `find_by(f)`, `index_of`, `insert`, `pop`, `remove`, `extend`, `min_by`/`max_by`, `sum_by`, descending sort.
- Maps: `entries`, `from_entries`, `merge` (and `map + map`), `get(k, default)`, `delete`, `map_values`, `filter` on maps, `invert`.

### Set operations
- `union`, `intersect`, `difference`, `is_subset` on lists (order kept, duplicates dropped).

### Copying
- `copy` (shallow) and `deep_copy`, since lists and maps are shared by reference.

## Session 3: units, dates & the network

The `units` and `dates` modules, plus everything that uses `ureq`. This is the heaviest session; if it needs splitting, do units + network first and dates second.

### Physical constants as quantities
- `c`, `g0`, `G`, `h`, `hbar`, `k_B`, `N_A`, `e_charge`, `m_e`, `R`, each with units: `1 kg * c^2 to kWh`. Variables still shadow them.

### SI prefix auto-scaling
- `to auto`: `0.0042 A` → `4.2 mA`, `1536000 B` → `1.46 MiB`.

### Missing unit categories
- Density, flow rate, torque, radiation (`Gy Sv`), typography (`pt px em`), cooking (`stick`, `pinch`).
- Fuel economy (`mpg` ↔ `L/100km`) needs inverse conversion.

### Historical currency rates
- `100 USD to EUR at date("2020-01-01")` via frankfurter's dated lookups. Cache each date's rates permanently, since they never change.

### HTTP GET
- `fetch(url)` returns the body; `.json` once JSON lands. Add a timeout and an error on non-2xx responses.

### Time zone abbreviations and city names
- `now to PST`, `now to tokyo`, `now to "new york"`. Abbreviations map to one IANA zone (ambiguous ones are documented); city names come from the IANA zone list plus an alias table.

### Format presets
- `to iso`, `to rfc2822`, `to rfc3339`, `to http_date`, `to time`, registered as `to` targets.

### More date parsing
- "the 3rd of March", "March 3rd", "Q3 2026", "week 42", "this weekend", "EOD", "in 2 business days".

### Sunrise/sunset and moon phase
- `sunrise(date, lat, lon)`, `sunset(...)`, `moon_phase(date)` (name + illumination %). Pure computation (NOAA equations).

### Countdown / stopwatch
- `timer(5 min)`: a live countdown with a bell. `stopwatch()`: returns the elapsed time when you press Enter. Also `sleep(d)`.

## Session 4: REPL, CLI & errors

Completion, hints and "did you mean" all draw on the same name lists (builtins, variables, units, `to` targets), so they belong together.

### Did you mean + type errors
- Do these first, since they build the shared name lookup. Close matches for unknown variables, functions, methods, units and targets: `undefined variable "uper"; did you mean upper?`.
- Builtin argument errors show the `Doc` signature: `round: expected round(x, digits?), got (str, int)`.

### Tab completion
- Names, units and `to` targets; after `.`, methods valid for a known variable's type. Uses rustyline's `Completer`.

### Syntax highlighting
- Uses the existing lexer through rustyline's `Highlighter`, with bracket matching.

### Hints
- Grey inline preview of the result while typing, via rustyline's `Hinter`. Skip anything with side effects (`print`, `write_file`, `fetch`, `rand`, assignments).

### History references
- `_1`, `_2`, ... for earlier results; `_` stays the last result.

### Multi-line editing
- Auto-indent inside `{ ... }`; a multi-line entry is one history item.

### Startup rc file
- `~/.config/zil/init.zil` (or `%APPDATA%\zil\init.zil`) runs before the REPL and scripts; `--no-rc` skips it.

### Implicit -e
- `zil 5 km to mi`: if the first argument isn't an existing file, join all arguments and evaluate them.

### Clipboard
- `clip` reads the clipboard and `v to clip` writes it. Uses `arboard`, or calls out to system tools (`pbcopy`, `clip.exe`, `wl-copy`/`xclip`).
