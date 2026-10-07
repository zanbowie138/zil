# text.translation

translate text between languages online; cached translations work offline

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### translate

```zil
# allow_network_access() first; cached translations work without it
# languages by code (es, de, zh-CN) or English name (spanish, german, chinese)
# from is auto-detected when left out
```

## Functions

| function | description |
|---|---|
| [`translate(text: str, to: str, from?: str)`](#translate) | translate text into another language by code or English name; the source is detected when omitted |

### translate

`translate(text: str, to: str, from?: str)`: translate text into another language by code or English name; the source is detected when omitted

```zil
"good morning".translate("es")
translate("Guten Morgen", "english", "german")
```

See also: [allow_network_access](../sys.md#allow_network_access)
