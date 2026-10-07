# art.frames

boxes, cowsay, and art side by side, stacked, overlaid, mirrored, rotated, scaled or inverted

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`boxed(s: str, style?: str)`](#boxed) | draw a box around text; style is "single" (default), "double", "round", "heavy", "ascii", "stars" or "shadow" |
| [`cowsay(s: str, critter?: str)`](#cowsay) | a critter saying s in a speech bubble; critter is "cow" (default), "cat", "ghost" or "crab" |
| [`think(s: str, critter?: str)`](#think) | like cowsay, in a thought bubble |
| [`beside(a: str, b: str, gap?: int)`](#beside) | put b to the right of a, tops aligned, gap spaces apart (default 1) |
| [`stack(a: str, b: str, align?: str)`](#stack) | put b under a, aligned "left" (default), "center" or "right" |
| [`overlay(a: str, b: str, x: int, y: int)`](#overlay) | draw b over a with its top-left at column x, row y; spaces in b are see-through |
| [`mirror(s: str, axis?: str)`](#mirror) | flip left-right ("h", default) or upside down ("v"), turning slashes and brackets to match |
| [`rotate(s: str, degrees: int)`](#rotate) | turn art clockwise by 90, 180 or 270 degrees |
| [`scale(s: str, n: int)`](#scale) | make every character an n×n block of itself |
| [`negative(s: str)`](#negative) | swap ink and background: spaces become blocks, blocks and braille dots invert |
| [`trim_art(s: str)`](#trim_art) | drop blank lines at the ends, shared left indentation and trailing spaces |

### boxed

`boxed(s: str, style?: str)`: draw a box around text; style is "single" (default), "double", "round", "heavy", "ascii", "stars" or "shadow"

```zil
boxed("hello")
# → "┌───────┐\n│ hello │\n└───────┘"
"zil\ncalc".boxed("round")
# → "╭──────╮\n│ zil  │\n│ calc │\n╰──────╯"
```

See also: [cowsay](../art/frames.md#cowsay), [stack](../art/frames.md#stack)

### cowsay

`cowsay(s: str, critter?: str)`: a critter saying s in a speech bubble; critter is "cow" (default), "cat", "ghost" or "crab"

```zil
cowsay("moo")
# → " _____\n< moo >\n -----\n   \\\n    \\\n         (__)\n         (oo)\n  /-------\\/\n / |     ||\n*  ||----||\n   ~~    ~~"
```

See also: [think](../art/frames.md#think), [boxed](../art/frames.md#boxed)

### think

`think(s: str, critter?: str)`: like cowsay, in a thought bubble

```zil
think("hmm", "ghost")
# → " _____\n( hmm )\n -----\n   o\n    o\n      .-.\n     (o o)\n     | O \\\n      \\   \\\n       `~~~'"
```

See also: [cowsay](../art/frames.md#cowsay)

### beside

`beside(a: str, b: str, gap?: int)`: put b to the right of a, tops aligned, gap spaces apart (default 1)

```zil
"a\nb".beside("c\nd\ne")
# → "a c\nb d\n  e"
```

See also: [stack](../art/frames.md#stack), [overlay](../art/frames.md#overlay)

### stack

`stack(a: str, b: str, align?: str)`: put b under a, aligned "left" (default), "center" or "right"

```zil
"wide one".stack("x", "right")
# → "wide one\n       x"
```

See also: [beside](../art/frames.md#beside)

### overlay

`overlay(a: str, b: str, x: int, y: int)`: draw b over a with its top-left at column x, row y; spaces in b are see-through

```zil
boxed("     ").overlay("hi", 2, 1)
# → "┌───────┐\n│ hi    │\n└───────┘"
```

See also: [beside](../art/frames.md#beside)

### mirror

`mirror(s: str, axis?: str)`: flip left-right ("h", default) or upside down ("v"), turning slashes and brackets to match

```zil
"(=^.^=)/".mirror
# → "\\(=^.^=)"
"/\\\n\\/".mirror("v")
# → "/\\\n\\/"
```

See also: [rotate](../art/frames.md#rotate), [flip](../fun/mangle.md#flip)

### rotate

`rotate(s: str, degrees: int)`: turn art clockwise by 90, 180 or 270 degrees

```zil
"abc\ndef".rotate(90)
# → "da\neb\nfc"
```

See also: [mirror](../art/frames.md#mirror)

### scale

`scale(s: str, n: int)`: make every character an n×n block of itself

```zil
"ab".scale(2)
# → "aabb\naabb"
```

See also: [banner](../art/figlet.md#banner)

### negative

`negative(s: str)`: swap ink and background: spaces become blocks, blocks and braille dots invert

```zil
"▀ █".negative
# → "▄█ "
```

See also: [invert](../data/maps.md#invert)

### trim_art

`trim_art(s: str)`: drop blank lines at the ends, shared left indentation and trailing spaces

```zil
"\n   ab\n    c\n\n".trim_art
# → "ab\n c"
```

See also: [dedent](../text/layout.md#dedent)

## More examples

### compose

```zil
# a caption under a picture
clipart("owl").stack("hoo?", "center")
# → " ,_,\n(O,O)\n(   )\n-\"-\"-\nhoo?"
# two friends
clipart("cat").beside(clipart("dog"), 4)
# → " /\\_/\\       __      _\n( o.o )    o'')}____//\n > ^ <      `_/      )\n            (_(_/-(_/"
# a cat says
cowsay("deploy on friday?", "cat")
# → " ___________________\n< deploy on friday? >\n -------------------\n   \\\n    \\\n      /\\_/\\\n     ( o.o )\n      > ^ <"
# a picture frame
clipart("house").boxed("double").boxed("shadow")
# → "┌──────────────┐\n│ ╔══════════╗ │▒\n│ ║     /\\   ║ │▒\n│ ║    /  \\  ║ │▒\n│ ║   /____\\ ║ │▒\n│ ║   | [] | ║ │▒\n│ ║   |_||_| ║ │▒\n│ ╚══════════╝ │▒\n└──────────────┘▒\n ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒"
# a dark sky
starfield(20, 4, 1).negative
# → "████ █████ █ ███████\n██████████████ █████\n█████ ██████ ██████ "
```
