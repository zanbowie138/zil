# art.images

PNG, JPEG and GIF images as ASCII, truecolor half blocks or dithered braille

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### modes

```zil
# "blocks" (default): two pixels per character in full color, for truecolor terminals
# "ascii": brightness as " .:-=+*#%@", pastes anywhere
# "braille": Floyd–Steinberg dithered dots, the sharpest in black and white
```

## Functions

| function | description |
|---|---|
| [`img(path: str, width?: int, mode?: str)`](#img) | draw an image file width characters wide (default: the terminal's, or 80); mode is "blocks" (default), "ascii" or "braille" |

### img

`img(path: str, width?: int, mode?: str)`: draw an image file width characters wide (default: the terminal's, or 80); mode is "blocks" (default), "ascii" or "braille"

```zil
img("cat.png")
img("logo.jpg", 40, "ascii")
img("photo.png", 60, "braille").negative
```

See also: [negative](../art/frames.md#negative), [boxed](../art/frames.md#boxed)
