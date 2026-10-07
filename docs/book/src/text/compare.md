# text.compare

edit distance, similarity, the closest of a list, line-by-line diffs

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`distance(a: str, b: str)`](#distance) | edit distance: inserts, deletes, substitutions and swaps of neighbors to turn a into b |
| [`similarity(a: str, b: str)`](#similarity) | 1 - distance / longer length: 1 is identical, 0 shares nothing |
| [`closest(s: str, options: list)`](#closest) | the option with the smallest edit distance to s (ignoring case); nil for an empty list |
| [`text_diff(a: str, b: str)`](#text_diff) | line diff: removed lines start with -, added with +, kept with two spaces |

### distance

`distance(a: str, b: str)`: edit distance: inserts, deletes, substitutions and swaps of neighbors to turn a into b

```zil
distance("kitten", "sitting")
# → 3
distance("form", "from")
# → 1
```

See also: [similarity](../text/compare.md#similarity), [closest](../text/compare.md#closest)

### similarity

`similarity(a: str, b: str)`: 1 - distance / longer length: 1 is identical, 0 shares nothing

```zil
similarity("color", "colour")
# → 0.833333
```

See also: [distance](../text/compare.md#distance)

### closest

`closest(s: str, options: list)`: the option with the smallest edit distance to s (ignoring case); nil for an empty list

```zil
"aple".closest(["apple", "maple", "ape"])
# → "apple"
```

See also: [distance](../text/compare.md#distance)

### text_diff

`text_diff(a: str, b: str)`: line diff: removed lines start with -, added with +, kept with two spaces

```zil
text_diff("a\nb\nc", "a\nc\nd")
# → "  a\n- b\n  c\n+ d"
```

See also: [distance](../text/compare.md#distance)

## More examples

### compare

```zil
# typo fixer
"recieve".closest(["deceive", "receive", "recipe"])
# → "receive"
# how alike
similarity("color", "colour").percent(0)
# → "83%"
# what changed
text_diff("a\nb\nc", "a\nB\nc\nd")
# → "  a\n- b\n+ B\n  c\n+ d"
```
