# art.renderers

nested data as a tree, heatmaps, progress bars, QR codes, chess boards from FEN, playing cards, dice faces, seven-segment digits

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`tree_view(v: map\|list)`](#tree_view) | nested maps and lists drawn like the `tree` command |
| [`heatmap(rows: list)`](#heatmap) | a list of rows of numbers as shades from ░ (low) to █ (high) |
| [`progress(frac: num, width?: int)`](#progress) | a progress bar for frac from 0 to 1, width cells wide (default 30) |
| [`qr(text: str, ec?: str)`](#qr) | a scannable QR code in half blocks, light on dark; ec is the error correction level "L", "M" (default), "Q" or "H" |
| [`chess(fen?: str)`](#chess) | a chess board from the piece part of a FEN string; the starting position by default |
| [`card(c: str)`](#card) | a playing card like "A♠", "10h" or "qd" (suits ♠♥♦♣ or s h d c); "?" is the back |
| [`cards(hand: list)`](#cards) | playing cards side by side |
| [`dice(faces: int\|list)`](#dice) | die faces with pips, 1-6, side by side for a list |
| [`segments(s: str\|int)`](#segments) | big seven-segment digits for 0-9, A-F, - . : and spaces |

### tree_view

`tree_view(v: map|list)`: nested maps and lists drawn like the `tree` command

```zil
tree_view({src: {main: "rs", lib: "rs"}, docs: ["book"]})
# → "├── src\n│   ├── main: rs\n│   └── lib: rs\n└── docs\n    └── book"
```

See also: [keys](../data/maps.md#keys), [from_json](../fs.md#from_json)

### heatmap

`heatmap(rows: list)`: a list of rows of numbers as shades from ░ (low) to █ (high)

```zil
[[1, 2, 3], [4, 5, 6], [7, 8, 9]].heatmap
# → "  ░░░░\n▒▒▒▒▓▓\n▓▓████"
```

See also: [bars](../data/charts.md#bars), [sparkline](../data/charts.md#sparkline)

### progress

`progress(frac: num, width?: int)`: a progress bar for frac from 0 to 1, width cells wide (default 30)

```zil
progress(0.42)
# → "▕████████████▋                 ▏ 42%"
progress(2 / 3, 12)
# → "▕████████    ▏ 67%"
```

See also: [bars](../data/charts.md#bars)

### qr

`qr(text: str, ec?: str)`: a scannable QR code in half blocks, light on dark; ec is the error correction level "L", "M" (default), "Q" or "H"

```zil
qr("zil")
# → "█████████████████████████████\n█████████████████████████████\n████ ▄▄▄▄▄ █▄▀▄███ ▄▄▄▄▄ ████\n████ █   █ █▀▄██ █ █   █ ████\n████ █▄▄▄█ █ █ ▀▄█ █▄▄▄█ ████\n████▄▄▄▄▄▄▄█ █▄▀ █▄▄▄▄▄▄▄████\n████ ▀██ █▄▄▄█▀ ▀▄▄ ▄▄█▀▄████\n████▀▄▄▀▀▀▄ █▄▀▄█▀ ▄ ██▄▀████\n████████▄█▄█  ██   ██▀█▄█████\n████ ▄▄▄▄▄ █▄█ ▀ ▄█▀█ █ ▀████\n████ █   █ █▄ ▀ ▀█▄ ▄▀▄  ████\n████ █▄▄▄█ ██▄▀▄█▀ ▄ ▄ ██████\n████▄▄▄▄▄▄▄█▄▄▄█▄▄▄███▄█▄████\n█████████████████████████████\n▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀"
```

See also: [url_build](../dev.md#url_build)

### chess

`chess(fen?: str)`: a chess board from the piece part of a FEN string; the starting position by default

```zil
chess()
# → "8 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜\n7 ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟\n6   ·   ·   ·   ·\n5 ·   ·   ·   ·  \n4   ·   ·   ·   ·\n3 ·   ·   ·   ·  \n2 ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙\n1 ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖\n  a b c d e f g h"
chess("8/8/8/4k3/8/8/4P3/4K3 w - - 0 1")
# → "8   ·   ·   ·   ·\n7 ·   ·   ·   ·  \n6   ·   ·   ·   ·\n5 ·   ·   ♚   ·  \n4   ·   ·   ·   ·\n3 ·   ·   ·   ·  \n2   ·   · ♙ ·   ·\n1 ·   ·   ♔   ·  \n  a b c d e f g h"
```

See also: [card](../art/renderers.md#card)

### card

`card(c: str)`: a playing card like "A♠", "10h" or "qd" (suits ♠♥♦♣ or s h d c); "?" is the back

```zil
card("Q♥")
# → "┌─────┐\n│Q    │\n│  ♥  │\n│    Q│\n└─────┘"
```

See also: [cards](../art/renderers.md#cards), [dice](../art/renderers.md#dice)

### cards

`cards(hand: list)`: playing cards side by side

```zil
cards(["As", "Kh", "?"])
# → "┌─────┐ ┌─────┐ ┌─────┐\n│A    │ │K    │ │░░░░░│\n│  ♠  │ │  ♥  │ │░░░░░│\n│    A│ │    K│ │░░░░░│\n└─────┘ └─────┘ └─────┘"
```

See also: [card](../art/renderers.md#card)

### dice

`dice(faces: int|list)`: die faces with pips, 1-6, side by side for a list

```zil
dice(5)
# → "┌───────┐\n│ ●   ● │\n│   ●   │\n│ ●   ● │\n└───────┘"
dice([1, 2, 3])
# → "┌───────┐ ┌───────┐ ┌───────┐\n│       │ │     ● │ │ ●     │\n│   ●   │ │       │ │   ●   │\n│       │ │ ●     │ │     ● │\n└───────┘ └───────┘ └───────┘"
```

See also: [roll](../fun/games.md#roll), [card](../art/renderers.md#card)

### segments

`segments(s: str|int)`: big seven-segment digits for 0-9, A-F, - . : and spaces

```zil
segments("12:45")
# → "      ▄▄          ▄▄\n   █  ▄▄█ • █▄▄█ █▄▄\n   █ █▄▄  •    █  ▄▄█"
segments("C0FFEE")
# → " ▄▄   ▄▄   ▄▄   ▄▄   ▄▄   ▄▄\n█    █  █ █▄▄  █▄▄  █▄▄  █▄▄\n█▄▄  █▄▄█ █    █    █▄▄  █▄▄"
```

See also: [banner](../art/figlet.md#banner), [clock](../time/zones.md#clock)

## More examples

### draw data

```zil
# a config at a glance
tree_view({server: {host: "localhost", ports: [80, 443]}, debug: false})
# → "├── server\n│   ├── host: localhost\n│   └── ports\n│       ├── 80\n│       └── 443\n└── debug: false"
# a multiplication table
(1..=6).map(|r| (1..=10).map(|c| r * c)).heatmap
# → "                ░░░░\n        ░░░░░░░░░░░░\n    ░░░░░░░░░░▒▒▒▒▒▒\n    ░░░░░░▒▒▒▒▒▒▒▒▓▓\n  ░░░░░░▒▒▒▒▒▒▓▓▓▓▓▓\n  ░░░░▒▒▒▒▒▒▓▓▓▓████"
# a link for your phone
qr("https://github.com/zanbowie138/zil")
# → "█████████████████████████████████████\n█████████████████████████████████████\n████ ▄▄▄▄▄ █▀  ▀█▀▀▄   ███ ▄▄▄▄▄ ████\n████ █   █ █ ▄▀█▀▄█▄▀▀▄  █ █   █ ████\n████ █▄▄▄█ ██ ▀████ ▄ ▄▄ █ █▄▄▄█ ████\n████▄▄▄▄▄▄▄█ █ █ █ ▀ █▄█▄█▄▄▄▄▄▄▄████\n████ ▀▀█▀█▄▀ ▄█▀▄▀██▀▄█▀█ ▄▀▀▄  █████\n████▀█▄ ▀▀▄▀ ▄▀█▀ ▄▄▀▄█▀█▀██▄▀███████\n████▄██▀▄▀▄▄ ▀▄▀ ▄█▀ ▀▀▀▀█  ▀██▀ ████\n█████▄▀▄█ ▄█▄ █ ███▄ █▄▀▄▄█▀ ▄ █▀████\n████ █▀ ▀▄▄▀▄▀▄▀█▀▀▀▀▄▀ █▄▀ ▀▄ ▀▀████\n████ ▄ █▀▄▄█▀▄██▀▀█ ▄▄█▄▀█▀█▄▄██▄████\n████▄██▄▄▄▄█ ▀█▀██ ▀▄▄▄▄ ▄▄▄ ▀▄▄▄████\n████ ▄▄▄▄▄ ██ ▄ ▀█▀  █▄  █▄█ ▄▄█▀████\n████ █   █ ██▀▄ ▀  ▀▄█▄   ▄▄▄ ▀▄ ████\n████ █▄▄▄█ ██▄ ▄▀▀▄▄▀▄▄▄▄▄  ▄  ▄▀████\n████▄▄▄▄▄▄▄█▄▄███▄██▄▄▄▄██▄▄▄█▄██████\n█████████████████████████████████████\n▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀"
# what you rolled
dice(roll("3d6").rolls)
# → "┌───────┐ ┌───────┐ ┌───────┐\n│ ●     │ │ ●     │ │ ●   ● │\n│   ●   │ │   ●   │ │   ●   │\n│     ● │ │     ● │ │ ●   ● │\n└───────┘ └───────┘ └───────┘"
# a royal flush
cards(["10♠", "J♠", "Q♠", "K♠", "A♠"])
# → "┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐\n│10   │ │J    │ │Q    │ │K    │ │A    │\n│  ♠  │ │  ♠  │ │  ♠  │ │  ♠  │ │  ♠  │\n│   10│ │    J│ │    Q│ │    K│ │    A│\n└─────┘ └─────┘ └─────┘ └─────┘ └─────┘"
# a big clock
segments(now.format("%H:%M"))
# → "             ▄▄   ▄▄\n   █ █▄▄█ • █▄▄  █▄▄█\n   █    █ •  ▄▄█ █▄▄█"
```
