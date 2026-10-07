# art.figlet

banners in block letters or FIGlet fonts: standard, slant, shadow, script, lean, banner, bubble and more, or any .flf file; 𝐛𝐨𝐥𝐝/𝒮𝒸𝓇𝒾𝓅𝓉/ｗｉｄｅ Unicode styles

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`banner(s: str, font?: str)`](#banner) | big letters; font is "block" (default, A-Z, 0-9, some punctuation), one of fonts(), or a path to a FIGlet .flf file |
| [`fonts()`](#fonts) | the built-in banner fonts |
| [`style(s: str, style: str)`](#style) | restyle letters and digits with Unicode look-alikes; style is one of styles() |
| [`styles()`](#styles) | the styles for style() |

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
# → ["block", "standard", "small", "slant", "big", "mini", "smslant", "shadow", "smshadow", "script", "smscript", "lean", "banner", "bubble", "digital"]
```

See also: [banner](../art/figlet.md#banner)

### style

`style(s: str, style: str)`: restyle letters and digits with Unicode look-alikes; style is one of styles()

```zil
"Hello".style("bold")
# → "𝐇𝐞𝐥𝐥𝐨"
"vaporwave".style("wide")
# → "ｖａｐｏｒｗａｖｅ"
```

See also: [styles](../art/figlet.md#styles), [banner](../art/figlet.md#banner), [fonts](../art/figlet.md#fonts)

### styles

`styles()`: the styles for style()

```zil
styles()
# → ["bold", "italic", "bold_italic", "script", "fraktur", "double", "sans", "sans_bold", "mono", "wide", "circled", "small_caps", "strike", "underline"]
```

See also: [style](../art/figlet.md#style)

## More examples

### banners

```zil
# a README header
banner("zil", "standard")
# → "     _ _\n ___(_) |\n|_  / | |\n / /| | |\n/___|_|_|"
# every font
fonts().map(|f| banner("Hi", f)).join("\n")
# → "██  ██  ██████\n██  ██    ██\n██████    ██\n██  ██    ██\n██  ██  ██████\n _   _ _\n| | | (_)\n| |_| | |\n|  _  | |\n|_| |_|_|\n _  _ _\n| || (_)\n| __ | |\n|_||_|_|\n    __  ___\n   / / / (_)\n  / /_/ / /\n / __  / /\n/_/ /_/_/\n _    _ _\n| |  | (_)\n| |__| |_\n|  __  | |\n| |  | | |\n|_|  |_|_|\n\n|_|o\n| ||\n   __ ___\n  / // (_)\n / _  / /\n/_//_/_/\n |   |_)\n |   | |\n ___ | |\n_|  _|_|\n |  |_)\n __ | |\n_| _|_|\n ,\n/|   | o\n |___|\n |   |\\|\n |   |/|_/\n ,\n/|  | o\n |--| |\n |  |)|/\n\n    _/    _/  _/\n   _/    _/\n  _/_/_/_/  _/\n _/    _/  _/\n_/    _/  _/\n#     #\n#     # #\n#     # #\n####### #\n#     # #\n#     # #\n#     # #\n  _   _\n / \\ / \\\n( H | i )\n \\_/ \\_/\n+-+-+\n|H|i|\n+-+-+"
# every style
styles().map(|st| "Zil 2".style(st))
# → ["𝐙𝐢𝐥 𝟐", "𝑍𝑖𝑙 2", "𝒁𝒊𝒍 2", "𝒵𝒾𝓁 2", "ℨ𝔦𝔩 2", "ℤ𝕚𝕝 𝟚", "𝖹𝗂𝗅 𝟤", "𝗭𝗶𝗹 𝟮", "𝚉𝚒𝚕 𝟸", "Ｚｉｌ ２", "Ⓩⓘⓛ ②", "Zɪʟ 2", "Z\u{336}i\u{336}l\u{336} 2\u{336}", "Z\u{332}i\u{332}l\u{332} 2\u{332}"]
# a boxed title
banner("v2", "small").boxed("round")
# → "╭───────────╮\n│      ___  │\n│ __ _|_  ) │\n│ \\ V // /  │\n│  \\_//___| │\n╰───────────╯"
```
