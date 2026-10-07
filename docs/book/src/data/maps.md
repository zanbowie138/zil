# data.maps

keys, values, lookups, merging and transforming maps

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### operators

```zil
# map + map
{a: 1} + {a: 2, b: 3}
# → {a: 2, b: 3}
```

## Functions

| function | description |
|---|---|
| [`keys(m: map)`](#keys) | list of map keys |
| [`values(m: map)`](#values) | list of map values |
| [`get(m: map, k: any, default?: any)`](#get) | value at k, or default (nil if not given) |
| [`has(m: map, k: any)`](#has) | true if m has the key k |
| [`del(m: map, k: any)`](#del) | remove k in place and return the map |
| [`merge(a: map, b: map)`](#merge) | new map with both; b wins on shared keys, same as a + b |
| [`entries(m: map)`](#entries) | [key, value] pairs, same as list(m) |
| [`from_entries(xs: list)`](#from_entries) | map from [key, value] pairs; later keys win |
| [`invert(m: map) / invert(c: str\|list)`](#invert) | swap a map's keys and values (values become string keys), or the opposite color |
| [`map_values(m: map, f: fn)`](#map_values) | apply f to every value, keeping keys |
| [`filter_keys(m: map, f: fn)`](#filter_keys) | keep entries whose key f is truthy for |

### keys

`keys(m: map)`: list of map keys

```zil
{a: 1, b: 2}.keys
# → ["a", "b"]
```

See also: [values](../data/maps.md#values)

### values

`values(m: map)`: list of map values

```zil
{a: 1, b: 2}.values
# → [1, 2]
```

See also: [keys](../data/maps.md#keys)

### get

`get(m: map, k: any, default?: any)`: value at k, or default (nil if not given)

```zil
{a: 1}.get("a")
# → 1
{a: 1}.get("b", 0)
# → 0
```

See also: [has](../data/maps.md#has)

### has

`has(m: map, k: any)`: true if m has the key k

```zil
{a: 1}.has("a")
# → true
```

See also: [get](../data/maps.md#get), [contains](../data.md#contains)

### del

`del(m: map, k: any)`: remove k in place and return the map

```zil
{a: 1, b: 2}.del("a")
# → {b: 2}
```

See also: [merge](../data/maps.md#merge)

### merge

`merge(a: map, b: map)`: new map with both; b wins on shared keys, same as a + b

```zil
merge({a: 1, b: 2}, {b: 3})
# → {a: 1, b: 3}
```

See also: [del](../data/maps.md#del)

### entries

`entries(m: map)`: [key, value] pairs, same as list(m)

```zil
{a: 1, b: 2}.entries
# → [["a", 1], ["b", 2]]
```

See also: [from_entries](../data/maps.md#from_entries), [list](../core.md#list)

### from_entries

`from_entries(xs: list)`: map from [key, value] pairs; later keys win

```zil
[["a", 1], ["b", 2]].from_entries
# → {a: 1, b: 2}
zip(["x", "y"], [1, 2]).from_entries
# → {x: 1, y: 2}
```

See also: [entries](../data/maps.md#entries)

### invert

`invert(m: map) / invert(c: str|list)`: swap a map's keys and values (values become string keys), or the opposite color

```zil
{a: 1, b: 2}.invert
# → {1: "a", 2: "b"}
invert("navy")
# → "#ffff7f"
```

See also: [entries](../data/maps.md#entries), [grayscale](../dev/colors.md#grayscale)

### map_values

`map_values(m: map, f: fn)`: apply f to every value, keeping keys

```zil
{a: 1, b: 2}.map_values(|v| v * 10)
# → {a: 10, b: 20}
```

See also: [map](../data/lists.md#map), [filter](../data/lists.md#filter)

### filter_keys

`filter_keys(m: map, f: fn)`: keep entries whose key f is truthy for

```zil
{a: 1, bb: 2}.filter_keys(|k| k.len > 1)
# → {bb: 2}
```

See also: [filter](../data/lists.md#filter)

## More examples

### maps

```zil
# sum a map
{a: 1, b: 2}.values.sum
# → 3
# as [key, value] pairs
{a: 1, b: 2}.list
# → [["a", 1], ["b", 2]]
# defaults, overridden
{color: "red", size: 1} + {size: 3}
# → {color: "red", size: 3}
# loop over entries
for [k, v] in {a: 1, b: 2}.entries { print("{k}={v}") }
# → nil
```
