# art.figlet

banners in block letters or FIGlet fonts: standard, small, slant, big, or any .flf file

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`banner(s: str, font?: str)`](#banner) | big letters; font is "block" (default, A-Z, 0-9, some punctuation), one of fonts(), or a path to a FIGlet .flf file |
| [`fonts()`](#fonts) | the built-in banner fonts |

### banner

`banner(s: str, font?: str)`: big letters; font is "block" (default, A-Z, 0-9, some punctuation), one of fonts(), or a path to a FIGlet .flf file

```zil
banner("hi!")
# → "██  ██  ██████    ██\n██  ██    ██      ██\n██████    ██      ██\n██  ██    ██\n██  ██  ██████    ██"
banner("zil", "slant")
# → "        _ __\n ____  (_) /\n/_  / / / /\n / /_/ / /\n/___/_/_/"
```

See also: [fonts](../art/figlet.md#fonts), [segments](../art/renderers.md#segments)

### fonts

`fonts()`: the built-in banner fonts

```zil
fonts()
# → ["block", "standard", "small", "slant", "big"]
```

See also: [banner](../art/figlet.md#banner)

## More examples

### banners

```zil
# a README header
banner("zil", "standard")
# → "     _ _\n ___(_) |\n|_  / | |\n / /| | |\n/___|_|_|"
# every font
fonts().map(|f| banner("Hi", f)).join("\n")
# → "██  ██  ██████\n██  ██    ██\n██████    ██\n██  ██    ██\n██  ██  ██████\n _   _ _\n| | | (_)\n| |_| | |\n|  _  | |\n|_| |_|_|\n _  _ _\n| || (_)\n| __ | |\n|_||_|_|\n    __  ___\n   / / / (_)\n  / /_/ / /\n / __  / /\n/_/ /_/_/\n _    _ _\n| |  | (_)\n| |__| |_\n|  __  | |\n| |  | | |\n|_|  |_|_|"
# a boxed title
banner("v2", "small").boxed("round")
# → "╭───────────╮\n│      ___  │\n│ __ _|_  ) │\n│ \\ V // /  │\n│  \\_//___| │\n╰───────────╯"
```
