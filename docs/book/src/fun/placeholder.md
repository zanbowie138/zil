# fun.placeholder

lorem ipsum placeholder text

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`lorem(words?: int)`](#lorem) | the classic lorem ipsum paragraph, or its first n words (repeating past the end) |

### lorem

`lorem(words?: int)`: the classic lorem ipsum paragraph, or its first n words (repeating past the end)

```zil
lorem(5)
# → "Lorem ipsum dolor sit amet."
```

See also: [fortune](../fun.md#fortune)

## More examples

### placeholder

```zil
# a heading
lorem(3)
# → "Lorem ipsum dolor."
# how long is it?
lorem().split.len
# → 69
```
