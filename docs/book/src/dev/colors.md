# dev.colors

colors as "#rrggbb": RGB/HSL, mixing, lightening, WCAG contrast

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`color(c)`](#color) | a color as "#rrggbb" from a name, "#rgb", "#rrggbb" or [r, g, b] |
| [`rgb(r, g, b) / rgb(c)`](#rgb) | make a color from 0-255 channels, or get a color's channels |
| [`hsl(h, s, l) / hsl(c)`](#hsl) | make a color from hue (degrees), saturation and lightness (0-100), or get them |
| [`mix(a, b, t?)`](#mix) | blend from a (t = 0) to b (t = 1), halfway by default |
| [`lighten(c, amount)`](#lighten) | raise HSL lightness by amount (0-1) |
| [`darken(c, amount)`](#darken) | lower HSL lightness by amount (0-1) |
| [`invert(c)`](#invert) | the opposite color |
| [`grayscale(c)`](#grayscale) | the gray with the same perceived brightness |
| [`contrast(a, b)`](#contrast) | WCAG contrast ratio, 1 to 21; text wants 4.5+ |
| [`swatch(c)`](#swatch) | a colored block plus the hex, for truecolor terminals |

### color

`color(c)`: a color as "#rrggbb" from a name, "#rgb", "#rrggbb" or [r, g, b]

```zil
color("tomato")
# → "#ff6347"
color("#f80")
# → "#ff8800"
color([255, 0, 128])
# → "#ff0080"
```

See also: [rgb](../dev/colors.md#rgb), [hsl](../dev/colors.md#hsl)

### rgb

`rgb(r, g, b) / rgb(c)`: make a color from 0-255 channels, or get a color's channels

```zil
rgb(255, 136, 0)
# → "#ff8800"
"orange".rgb
# → [255, 165, 0]
```

See also: [hsl](../dev/colors.md#hsl), [color](../dev/colors.md#color)

### hsl

`hsl(h, s, l) / hsl(c)`: make a color from hue (degrees), saturation and lightness (0-100), or get them

```zil
hsl(210, 80, 50)
# → "#1980e6"
"teal".hsl
# → [180, 100, 25]
```

See also: [rgb](../dev/colors.md#rgb), [color](../dev/colors.md#color)

### mix

`mix(a, b, t?)`: blend from a (t = 0) to b (t = 1), halfway by default

```zil
mix("red", "blue")
# → "#800080"
mix("white", "black", 25%)
# → "#bfbfbf"
```

See also: [lighten](../dev/colors.md#lighten), [darken](../dev/colors.md#darken)

### lighten

`lighten(c, amount)`: raise HSL lightness by amount (0-1)

```zil
lighten("#3478f6", 20%)
# → "#96b9fa"
```

See also: [darken](../dev/colors.md#darken), [mix](../dev/colors.md#mix)

### darken

`darken(c, amount)`: lower HSL lightness by amount (0-1)

```zil
darken("#3478f6", 20%)
# → "#0847bc"
```

See also: [lighten](../dev/colors.md#lighten), [mix](../dev/colors.md#mix)

### invert

`invert(c)`: the opposite color

```zil
invert("navy")
# → "#ffff7f"
```

See also: [grayscale](../dev/colors.md#grayscale)

### grayscale

`grayscale(c)`: the gray with the same perceived brightness

```zil
grayscale("tomato")
# → "#969696"
```

See also: [invert](../dev/colors.md#invert)

### contrast

`contrast(a, b)`: WCAG contrast ratio, 1 to 21; text wants 4.5+

```zil
contrast("white", "black")
# → 21
contrast("#777", "white")
# → 4.48
```

See also: [swatch](../dev/colors.md#swatch)

### swatch

`swatch(c)`: a colored block plus the hex, for truecolor terminals

```zil
swatch("tomato")
# → "\u{1b}[48;2;255;99;71m    \u{1b}[0m #ff6347"
["red", "gold", "teal"].map(swatch).join(" ")
# → "\u{1b}[48;2;255;0;0m    \u{1b}[0m #ff0000 \u{1b}[48;2;255;215;0m    \u{1b}[0m #ffd700 \u{1b}[48;2;0;128;128m    \u{1b}[0m #008080"
```

See also: [color](../dev/colors.md#color)

## More examples

### colors

```zil
# named to hex
color("rebeccapurple")
# → "#663399"
# halfway between
mix("red", "blue")
# → "#800080"
# hover shade
darken("#3478f6", 10%)
# → "#0a5aed"
# readable text?
contrast("white", "#777")
# → 4.48
# a palette
(0..5).map(|i| hsl(i * 72, 70, 50))
# → ["#d92626", "#b5d926", "#26d96e", "#266ed9", "#b526d9"]
```
