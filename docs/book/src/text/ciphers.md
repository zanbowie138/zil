# text.ciphers

Caesar, ROT13, Atbash, Vigenère, Morse, Braille (for fun, not secrecy)

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`caesar(s: str, shift: int)`](#caesar) | shift each letter by shift places; negative shifts decode |
| [`rot13(s: str)`](#rot13) | Caesar by 13; applying it twice gives the original |
| [`atbash(s: str)`](#atbash) | mirror the alphabet: a <-> z, b <-> y |
| [`vigenere(s: str, key: str)`](#vigenere) | Caesar with a shift per letter from a key word |
| [`unvigenere(s: str, key: str)`](#unvigenere) | undo vigenere |
| [`morse(s: str)`](#morse) | text to Morse code, or Morse (dots, dashes, / between words) back to text |
| [`braille(s: str)`](#braille) | text to Grade 1 Braille cells, or Braille back to text |

### caesar

`caesar(s: str, shift: int)`: shift each letter by shift places; negative shifts decode

```zil
"hello".caesar(3)
# → "khoor"
"khoor".caesar(-3)
# → "hello"
```

See also: [rot13](../text/ciphers.md#rot13), [vigenere](../text/ciphers.md#vigenere)

### rot13

`rot13(s: str)`: Caesar by 13; applying it twice gives the original

```zil
"hello".rot13
# → "uryyb"
```

See also: [caesar](../text/ciphers.md#caesar)

### atbash

`atbash(s: str)`: mirror the alphabet: a <-> z, b <-> y

```zil
"wizard".atbash
# → "draziw"
```

See also: [caesar](../text/ciphers.md#caesar)

### vigenere

`vigenere(s: str, key: str)`: Caesar with a shift per letter from a key word

```zil
"attack at dawn".vigenere("lemon")
# → "lxfopv ef rnhr"
```

See also: [unvigenere](../text/ciphers.md#unvigenere), [caesar](../text/ciphers.md#caesar)

### unvigenere

`unvigenere(s: str, key: str)`: undo vigenere

```zil
"lxfopv ef rnhr".unvigenere("lemon")
# → "attack at dawn"
```

See also: [vigenere](../text/ciphers.md#vigenere)

### morse

`morse(s: str)`: text to Morse code, or Morse (dots, dashes, / between words) back to text

```zil
"sos".morse
# → "... --- ..."
".... .. / - .... . .-. .".morse
# → "HI THERE"
```

See also: [braille](../text/ciphers.md#braille)

### braille

`braille(s: str)`: text to Grade 1 Braille cells, or Braille back to text

```zil
"hello world".braille
# → "⠓⠑⠇⠇⠕⠀⠺⠕⠗⠇⠙"
"⠓⠊".braille
# → "hi"
```

See also: [morse](../text/ciphers.md#morse)

## More examples

### ciphers

```zil
# SOS
"sos".morse
# → "... --- ..."
# and back
"... --- ...".morse
# → "SOS"
# spoilers
"the butler did it".rot13
# → "gur ohgyre qvq vg"
# crack a Caesar
(1..26).map(|k| "Wkh hdjoh odqgv".caesar(-k)).grep("eagle")
# → ["The eagle lands"]
# Vigenère round trip
"attack at dawn".vigenere("lemon").unvigenere("lemon")
# → "attack at dawn"
# Braille
"hello".braille
# → "⠓⠑⠇⠇⠕"
```
