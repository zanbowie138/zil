# art

clip art, rainbows and gradients; boxes, cowsay and composition, fractals, QR codes, chess boards, FIGlet banners, images and animation below

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Submodules

| module | about |
|---|---|
| [frames](art/frames.md) | boxes, cowsay, and art side by side, stacked, overlaid, mirrored, rotated, scaled or inverted |
| [generate](art/generate.md) | L-systems and named fractals in braille, cellular automata like rule 30, the Ulam prime spiral, starfields |
| [renderers](art/renderers.md) | nested data as a tree, heatmaps, progress bars, QR codes, chess boards from FEN, playing cards, dice faces, seven-segment digits |
| [figlet](art/figlet.md) | banners in block letters or FIGlet fonts: standard, slant, shadow, script, lean, banner, bubble and more, or any .flf file; 𝐛𝐨𝐥𝐝/𝒮𝒸𝓇𝒾𝓅𝓉/ｗｉｄｅ Unicode styles |
| [images](art/images.md) | PNG, JPEG and GIF images as ASCII, truecolor half blocks or dithered braille |
| [motion](art/motion.md) | play a list of frames, or a function of the frame number, as an animation in the terminal |
| [toys](art/toys.md) | ASCII Mandelbrot set, Conway's Game of Life, random mazes |

## Functions

| function | description |
|---|---|
| [`clipart(name?: str)`](#clipart) | a piece from the gallery, or the list of names |
| [`rainbow(s: str)`](#rainbow) | color each character along a diagonal rainbow, for truecolor terminals |
| [`gradient(s: str, from: str\|list, to: str\|list)`](#gradient) | fade text left to right between two colors, for truecolor terminals |
| [`strip_ansi(s: str)`](#strip_ansi) | remove terminal color and cursor codes |

### clipart

`clipart(name?: str)`: a piece from the gallery, or the list of names

```zil
clipart("owl")
# → " ,_,\n(O,O)\n(   )\n-\"-\"-"
clipart().len
# → 17
```

See also: [cowsay](art/frames.md#cowsay), [boxed](art/frames.md#boxed)

### rainbow

`rainbow(s: str)`: color each character along a diagonal rainbow, for truecolor terminals

```zil
"rainbow".rainbow
# → "\u{1b}[38;2;255;51;51mr\u{1b}[38;2;255;92;51ma\u{1b}[38;2;255;133;51mi\u{1b}[38;2;255;173;51mn\u{1b}[38;2;255;214;51mb\u{1b}[38;2;255;255;51mo\u{1b}[38;2;214;255;51mw\u{1b}[0m"
```

See also: [gradient](art.md#gradient), [strip_ansi](art.md#strip_ansi)

### gradient

`gradient(s: str, from: str|list, to: str|list)`: fade text left to right between two colors, for truecolor terminals

```zil
gradient("sunset", "gold", "crimson")
# → "\u{1b}[38;2;255;215;0ms\u{1b}[38;2;248;176;12mu\u{1b}[38;2;241;137;24mn\u{1b}[38;2;234;98;36ms\u{1b}[38;2;227;59;48me\u{1b}[38;2;220;20;60mt\u{1b}[0m"
```

See also: [rainbow](art.md#rainbow), [color](dev/colors.md#color)

### strip_ansi

`strip_ansi(s: str)`: remove terminal color and cursor codes

```zil
gradient("ab", "red", "blue").strip_ansi
# → "ab"
```

See also: [rainbow](art.md#rainbow)

## More examples

### gallery

```zil
# what's in it
clipart()
# → ["cat", "dog", "owl", "bunny", "fish", "duck", "bat", "snail", "crab", "ghost", "skull", "coffee", "rocket", "house", "tree", "cactus", "heart"]
# a cat in a box
clipart("cat").boxed("round")
# → "╭─────────╮\n│  /\\_/\\  │\n│ ( o.o ) │\n│  > ^ <  │\n╰─────────╯"
# pride
banner("zil", "slant").rainbow
# → "        \u{1b}[38;2;133;255;51m_ \u{1b}[38;2;51;255;51m_\u{1b}[38;2;51;255;92m_\u{1b}[0m\n \u{1b}[38;2;255;112;51m_\u{1b}[38;2;255;153;51m_\u{1b}[38;2;255;194;51m_\u{1b}[38;2;255;235;51m_  \u{1b}[38;2;153;255;51m(\u{1b}[38;2;112;255;51m_\u{1b}[38;2;71;255;51m) \u{1b}[38;2;51;255;112m/\u{1b}[0m\n\u{1b}[38;2;255;92;51m/\u{1b}[38;2;255;133;51m_  \u{1b}[38;2;255;255;51m/ \u{1b}[38;2;173;255;51m/ \u{1b}[38;2;92;255;51m/ \u{1b}[38;2;51;255;92m/\u{1b}[0m\n \u{1b}[38;2;255;153;51m/ \u{1b}[38;2;255;235;51m/\u{1b}[38;2;235;255;51m_\u{1b}[38;2;194;255;51m/ \u{1b}[38;2;112;255;51m/ \u{1b}[38;2;51;255;71m/\u{1b}[0m\n\u{1b}[38;2;255;133;51m/\u{1b}[38;2;255;173;51m_\u{1b}[38;2;255;214;51m_\u{1b}[38;2;255;255;51m_\u{1b}[38;2;214;255;51m/\u{1b}[38;2;173;255;51m_\u{1b}[38;2;133;255;51m/\u{1b}[38;2;92;255;51m_\u{1b}[38;2;51;255;51m/\u{1b}[0m"
# measure colored art
rainbow("hi").strip_ansi.len
# → 2
```
