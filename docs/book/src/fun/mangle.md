# fun.mangle

mock case, l33t, uwu, Pig Latin, zalgo, upside-down text, NATO spelling and emoji

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`mock(s: str)`](#mock) | aLtErNaTiNg CaSe, for quoting someone sarcastically |
| [`leet(s: str)`](#leet) | l33t sp34k |
| [`uwu(s: str)`](#uwu) | uwu-ify text |
| [`pig_latin(s: str)`](#pig_latin) | translate to Pig Latin, keeping capitals and punctuation |
| [`zalgo(s: str, marks?: int)`](#zalgo) | Z̷a̸l̵g̶o̴ text: pile marks (default 3) onto every character |
| [`flip(s: str)`](#flip) | turn text upside down |
| [`nato(s: str)`](#nato) | spell with the NATO phonetic alphabet |
| [`emoji(name: str)`](#emoji) | an emoji by name, or nil |
| [`emojify(s: str)`](#emojify) | replace :name: codes with emoji; unknown codes stay |

### mock

`mock(s: str)`: aLtErNaTiNg CaSe, for quoting someone sarcastically

```zil
"this is fine".mock
# → "tHiS iS fInE"
```

See also: [upper](../text.md#upper), [leet](../fun/mangle.md#leet)

### leet

`leet(s: str)`: l33t sp34k

```zil
"elite hacker".leet
# → "3l173 h4ck3r"
```

See also: [mock](../fun/mangle.md#mock)

### uwu

`uwu(s: str)`: uwu-ify text

```zil
"hello friend, I really love this".uwu
# → "hewwo fwiend, I weawwy wuv this ^w^"
```

See also: [mock](../fun/mangle.md#mock)

### pig_latin

`pig_latin(s: str)`: translate to Pig Latin, keeping capitals and punctuation

```zil
"Hello, string theory!".pig_latin
# → "Ellohay, ingstray eorythay!"
```

See also: [nato](../fun/mangle.md#nato)

### zalgo

`zalgo(s: str, marks?: int)`: Z̷a̸l̵g̶o̴ text: pile marks (default 3) onto every character

```zil
"he comes".zalgo
# → "h\u{34a}\u{361}\u{306}e\u{31c}\u{32b}\u{326} c\u{34d}\u{354}\u{321}o\u{34c}\u{31b}\u{32c}m\u{345}\u{359}\u{368}e\u{354}\u{329}\u{350}s\u{300}\u{33d}\u{32b}"
"ok".zalgo(1)
# → "o\u{325}k\u{34f}"
```

See also: [flip](../fun/mangle.md#flip)

### flip

`flip(s: str)`: turn text upside down

```zil
"hello world".flip
# → "plɹoʍ ollǝɥ"
```

See also: [reverse](../data.md#reverse), [zalgo](../fun/mangle.md#zalgo)

### nato

`nato(s: str)`: spell with the NATO phonetic alphabet

```zil
"SOS".nato
# → "Sierra Oscar Sierra"
"b2b".nato
# → "Bravo Two Bravo"
```

See also: [morse](../text/ciphers.md#morse)

### emoji

`emoji(name: str)`: an emoji by name, or nil

```zil
emoji("taco")
# → "🌮"
emoji("fire")
# → "🔥"
```

See also: [emojify](../fun/mangle.md#emojify)

### emojify

`emojify(s: str)`: replace :name: codes with emoji; unknown codes stay

```zil
":fire: deploy on friday :skull:".emojify
# → "🔥 deploy on friday 💀"
```

See also: [emoji](../fun/mangle.md#emoji)

## More examples

### mangle

```zil
# reply to a bad take
"tabs are better than spaces".mock
# → "tAbS aRe BeTtEr ThAn SpAcEs"
# spell it over the phone
"zil 2".nato
# → "Zulu India Lima / Two"
# table flip
"(╯°□°)╯ " + "zil".flip
# → "(╯°□°)╯ lᴉz"
# a commit message
":rocket: ship it :tada:".emojify
# → "🚀 ship it 🎉"
```
