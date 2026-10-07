# fun.placeholder

lorem ipsum placeholder text, random team and person names

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`lorem(words?: int)`](#lorem) | the classic lorem ipsum paragraph, or its first n words (repeating past the end) |
| [`team_name()`](#team_name) | a random team name, like "The Crimson Otters" |
| [`person_name()`](#person_name) | a random first and last name |

### lorem

`lorem(words?: int)`: the classic lorem ipsum paragraph, or its first n words (repeating past the end)

```zil
lorem(5)
# → "Lorem ipsum dolor sit amet."
```

See also: [fortune](../fun.md#fortune)

### team_name

`team_name()`: a random team name, like "The Crimson Otters"

```zil
team_name()
# → "The Undefeated Dragons"
```

See also: [person_name](../fun/placeholder.md#person_name)

### person_name

`person_name()`: a random first and last name

```zil
person_name()
# → "Taylor Tanaka"
```

See also: [team_name](../fun/placeholder.md#team_name)

## More examples

### placeholder

```zil
# a heading
lorem(3)
# → "Lorem ipsum dolor."
# how long is it?
lorem().split.len
# → 69
# name the squad
team_name()
# → "The Wild Llamas"
# a test user
person_name()
# → "Avery Chen"
```
