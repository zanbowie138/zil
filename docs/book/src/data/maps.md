# data.maps

keys and values of maps

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`keys(map)`](#keys) | list of map keys |
| [`values(map)`](#values) | list of map values |

### keys

`keys(map)`: list of map keys

```zil
{a: 1, b: 2}.keys
# → ["a", "b"]
```

See also: [values](../data/maps.md#values)

### values

`values(map)`: list of map values

```zil
{a: 1, b: 2}.values
# → [1, 2]
```

See also: [keys](../data/maps.md#keys)

## More examples

### maps

```zil
# sum a map
{a: 1, b: 2}.values.sum
# → 3
# as [key, value] pairs
{a: 1, b: 2}.list
# → [["a", 1], ["b", 2]]
```
