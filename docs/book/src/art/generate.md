# art.generate

L-systems and named fractals in braille, cellular automata like rule 30, the Ulam prime spiral, starfields

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`lsystem(axiom: str, rules: map, angle: num, n: int, width?: int)`](#lsystem) | rewrite axiom n times with rules (one character → its replacement), then draw it with a turtle in braille, width characters wide (default 60). F and G draw a step, f steps without drawing, + and - turn by angle degrees, \| turns around, [ and ] save and restore position; other characters do nothing. Starts facing up |
| [`fractal(name?: str, n?: int, width?: int)`](#fractal) | a named L-system fractal: "dragon", "koch", "hilbert", "sierpinski", "levy", "gosper", "tree", "fern" or "bush"; n is the depth, width as in lsystem. No name lists them |
| [`rule(n: int, width?: int, steps?: int)`](#rule) | an elementary cellular automaton, 0-255, grown from one cell for steps rows (default width / 2) on a wrapping row of width cells (default 64); two rows per line |
| [`spiral(size?: int)`](#spiral) | an Ulam spiral: 1, 2, 3… wound out from the center with the primes as dots, which line up on diagonals; size characters wide (default 40) |
| [`starfield(width: int, height: int, seed?: int)`](#starfield) | a random night sky; the same seed gives the same sky |

### lsystem

`lsystem(axiom: str, rules: map, angle: num, n: int, width?: int)`: rewrite axiom n times with rules (one character → its replacement), then draw it with a turtle in braille, width characters wide (default 60). F and G draw a step, f steps without drawing, + and - turn by angle degrees, | turns around, [ and ] save and restore position; other characters do nothing. Starts facing up

```zil
lsystem("F", {F: "F+F-F-F+F"}, 90, 3, 30)
# → "             ⡤⠇\n           ⣀⡏⣏⡇\n        ⢀⣀⣖⡃\n      ⢀⣰⠚ ⠓⣗⣆⣖⡆\n     ⢰⢺⢲   ⡖⡗⡗⡆\n   ⢠⠼⠉⠈⠉   ⠉⠁⠉⠁\n ⡤⠏⠹⠽\n⣏⡁\n ⠓⣆⣰⣲\n   ⠘⢲⣀⢀⣀   ⣀⡀⣀⡀\n     ⠸⢼⠼   ⠧⡧⡧⠇\n      ⠈⠹⢤ ⡤⡯⠏⠯⠇\n        ⠈⠉⠯⡅\n           ⠉⣇⣏⡇\n             ⠓⡆"
```

See also: [fractal](../art/generate.md#fractal)

### fractal

`fractal(name?: str, n?: int, width?: int)`: a named L-system fractal: "dragon", "koch", "hilbert", "sierpinski", "levy", "gosper", "tree", "fern" or "bush"; n is the depth, width as in lsystem. No name lists them

```zil
fractal("koch", 3, 30)
# → "          ⡀⠠⡴⢧⠄⢀\n        ⠐⡞⠑⠚  ⠓⠊⢓⡆\n   ⣀⣠⣀  ⢈⣙⠄    ⠠⣋⡁  ⣀⣠⣀\n⣀⡴⣀⡕ ⢪⣀⢦⣀⠇      ⠐⣅⡴⣀⡕ ⢪⣀⢦⣀\n⠯⡄                      ⢠⠽\n⢖⠋                      ⠙⡲\n⠓⠲⠒⡢                  ⢔⠒⠖⠚\n   ⣉⡧                ⠰⣉\n⡤⠴⠤⠕                  ⠪⠤⠦⢤\n⠮⣄                      ⣠⠵\n⣖⠃                      ⠘⣲\n⠉⠳⠉⡣ ⢜⠉⠞⠉⡆      ⠠⡋⠳⠉⡣ ⢜⠉⠞⠉\n   ⠉⠙⠉  ⢈⣩⠂    ⠐⣍⡁  ⠉⠙⠉\n        ⠠⢧⡠⢤  ⡤⢄⡬⠇\n          ⠁⠐⠳⡞⠂⠈"
fractal()
# → ["dragon", "koch", "hilbert", "sierpinski", "levy", "gosper", "tree", "fern", "bush"]
```

See also: [lsystem](../art/generate.md#lsystem), [rule](../art/generate.md#rule)

### rule

`rule(n: int, width?: int, steps?: int)`: an elementary cellular automaton, 0-255, grown from one cell for steps rows (default width / 2) on a wrapping row of width cells (default 64); two rows per line

```zil
rule(30, 32)
# → "               ▄█▄\n             ▄█▀▄▄█▄\n           ▄█▀▄▄█▄ ▄█▄\n         ▄█▀▄▄█▄  ▄█▄▄█▄\n       ▄█▀▄▄█▄ ▄█▀▀▄   ▄█▄\n     ▄█▀▄▄█▄  ▄█ █▀▀▀ █▀▄▄█▄\n   ▄█▀▄▄█▄ ▄█▀▀▄▄█▀▄▄█▀ █▄ ▄█▄\n ▄█▀▄▄█▄  ▄█ █▀▀▄ ▄█▄▄█▀▀▄▄█▄▄█▄"
rule(90, 32, 16)
# → "               ▄▀▄\n             ▄▀▄ ▄▀▄\n           ▄▀▄     ▄▀▄\n         ▄▀▄ ▄▀▄ ▄▀▄ ▄▀▄\n       ▄▀▄             ▄▀▄\n     ▄▀▄ ▄▀▄         ▄▀▄ ▄▀▄\n   ▄▀▄     ▄▀▄     ▄▀▄     ▄▀▄\n ▄▀▄ ▄▀▄ ▄▀▄ ▄▀▄ ▄▀▄ ▄▀▄ ▄▀▄ ▄▀▄"
```

See also: [life](../art/toys.md#life), [fractal](../art/generate.md#fractal)

### spiral

`spiral(size?: int)`: an Ulam spiral: 1, 2, 3… wound out from the center with the primes as dots, which line up on diagonals; size characters wide (default 40)

```zil
spiral(20)
# → "⢀⠄⠁⢐⢄⠄⠐ ⠔⠑⠄⠕   ⢁ ⠁⢐\n⢀⠄ ⢀⠁⢁  ⠁⠔⠁⢔⢐ ⠐⢄⠁⠄⠄⠁\n⠐⠑⢄⠁⢁⠄ ⠕⠁⢀ ⢅⢀⠐⢅⢐⢀ ⠐⠐\n⢀⠁⢁⠐⢀ ⢑⢀⠐⢑⠄⠄ ⢀⠅⢁⢄⠑⠄⠄\n ⠐⠄⠄⠄⠔⠄⠔⠔⢄⢁⢕⠐⠑ ⠄⠐⢐⠐\n  ⢀⠐⢀⢀ ⢑⢔⠑⢌⠁⢁⢁⠁⢀⠁⢁⢁\n⢀⢅  ⢁ ⢅⠄⠑ ⠄⠁⢄ ⠕⠁⢀⠁⠄⢄\n  ⠅⢄⠄⢅ ⠐⠑ ⠕⢅⢀⠐⠄⠐ ⠄⠔\n ⠑⠐⠁⠁⢀⠐⢅⠄ ⢅⠔⠁⢁ ⠐⠄ ⢐⢅\n⢐ ⢐⢀⠁⢁⠐ ⢁   ⠐ ⢐⠄ ⠑⠄"
```

See also: [is_prime](../math/numtheory.md#is_prime)

### starfield

`starfield(width: int, height: int, seed?: int)`: a random night sky; the same seed gives the same sky

```zil
starfield(30, 4, 7)
# → "           .\n    ·            ·\n            · ·   + +\n             . ·"
```

See also: [negative](../art/frames.md#negative)

## More examples

### grow things

```zil
# a dragon curve
fractal("dragon", 8, 30)
# → "                   ⣀⣸⣉⡇ ⢀⣸⣉⡇\n                   ⠓⢺⠒⡆⡖⢺⢺⠒⡆⡖⠂\n  ⠸⠭⡧⠏⠹⢤       ⢠⢤  ⠯⢽⠭⡯⡅⠈⠉ ⠉⠯⠅\n⣖⣲⣰⣒⡃ ⠐⠚      ⣖⣺⣚ ⣀⣖⣚ ⠓⠃   ⣆⡖⠂\n ⠸⢼⠤⠇         ⡤⢼⢼⠤⡧⡧⢼⠤⡄\n⣀⣀⢘⣒⣆⣖⣲  ⣀⣰⣲  ⣓⣺⣺⣒⣗⣗⣺⠒⣗⡆\n⠧⢼⢼⠤⡧⡧⢤⢠⠤⡧⢼⢤ ⡤⡧⢼⢼⠤⡧⡧⢤\n ⠈⠉ ⠉⠁⠈⢹⣉⣏⣹⣹⣉⡁⠉⠉⠈⠉⣏⣏⣹⣉⣇⡀\n     ⡖⢲⢰⠒⡗⠚⠘⠒⠃ ⢰⢲ ⡖⡗⠚ ⠓⠃\n     ⠉⠹⠽⠉⠯⠽    ⠈⠹⠭⠏⠯⠽"
# a tree in a box
fractal("tree", 6, 24).boxed("round")
# → "╭───────────────────╮\n│  ⢀⡀⠐⢾ ⡯⠂ ⠐⠪⡇⡹⠒ ⣀  │\n│ ⣀⠓⣇  ⢹     ⢹⠁ ⢀⠗⢃ │\n│ ⠞⠉⠉⠑⠤⣸     ⢸⡠⠔⠉⠉⠱ │\n│      ⠘⢆   ⢀⠎      │\n│       ⠈⢢ ⣠⠃       │\n│         ⢳⠃        │\n│         ⢸         │\n│         ⢸         │\n│         ⢸         │\n│         ⢸         │\n│         ⢸         │\n│         ⢸         │\n╰───────────────────╯"
# Sierpinski, two ways
rule(90, 32, 16).beside(fractal("sierpinski", 4, 16), 4)
# → "               ▄▀▄                         ⣰⣇\n             ▄▀▄ ▄▀▄                      ⣼⣷⣾⣧\n           ▄▀▄     ▄▀▄                  ⢀⡾⢿⡄⢠⡿⢷⡀\n         ▄▀▄ ▄▀▄ ▄▀▄ ▄▀▄               ⢠⣞⠛⠚⠛⠛⠓⠛⣳⡄\n       ▄▀▄             ▄▀▄            ⣠⣿⣻⣦    ⣴⣟⣿⣄\n     ▄▀▄ ▄▀▄         ▄▀▄ ▄▀▄         ⣰⣯⡉⠉⣹⣧⡀⢀⣼⣏⠉⢉⣽⣆\n   ▄▀▄     ▄▀▄     ▄▀▄     ▄▀▄      ⠼⠷⠼⠷⠼⠷⠼⠷⠾⠧⠾⠧⠾⠧⠾⠧\n ▄▀▄ ▄▀▄ ▄▀▄ ▄▀▄ ▄▀▄ ▄▀▄ ▄▀▄ ▄▀▄"
# invent a plant
lsystem("X", {X: "F[-X][X]F[-X]+FX", F: "FF"}, 25, 4, 30)
# → "⣀⡀⠐⣄⡀ ⠈⢧ ⡄\n ⠈⠉⠙⠳⡀ ⠈⢷⡇    ⠒⡄⢀\n     ⠈⠲⣄⢻⢆⣜    ⠸⣸\n        ⠹⣿⠎     ⡟  ⢰\n        ⡀⠹⣀   ⠢⡌⣇ ⢠⣏⡄\n        ⠉⠓⢵⣇  ⠠⡸⣿⣜⠟⡉\n           ⢻   ⣷⣿⡿⠊⢱\n           ⠈⢣ ⣬⡿⡇ ⢠⣏⡀\n             ⢣⡇⣧⣏⣼⡟⠋\n             ⢸⡝⢺⣵⣿⠖⠁\n             ⢸⢠⠋\n             ⢸⠃\n             ⢸\n             ⢸\n             ⢸"
```
