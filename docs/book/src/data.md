# data

length, search, sorting and picking across strings, lists and maps; lists and maps below

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### types

```zil
# list
[1, 2, 3]
# map
{a: 1, b: 2}
```

### operators

```zil
# x in v
2 in [1, 2]
# → true
```

## Submodules

| module | about |
|---|---|
| [lists](data/lists.md) | ranges, map/filter/reduce, building lists |
| [maps](data/maps.md) | keys and values of maps |

## Functions

| function | description |
|---|---|
| [`len(v: str\|list\|map)`](#len) | length of a string, list or map |
| [`contains(v: str\|list\|map, x: any)`](#contains) | substring/regex in a string, item in a list, key in a map |
| [`find(v: str\|list, x: any)`](#find) | index of the first match, or nil |
| [`count(v: str\|list, x: any)`](#count) | number of matches in a string or list |
| [`reverse(v: str\|list)`](#reverse) | reverse a string or list |
| [`sort(xs: list, key?: fn)`](#sort) | sorted copy, optionally by key function |
| [`unique(xs: list)`](#unique) | drop duplicates, keeping first occurrences |
| [`first(xs: list)`](#first) | first item, or nil |
| [`last(xs: list)`](#last) | last item, or nil |

### len

`len(v: str|list|map)`: length of a string, list or map

```zil
"héllo".len
# → 5
[1, 2, 3].len
# → 3
{a: 1}.len
# → 1
```

### contains

`contains(v: str|list|map, x: any)`: substring/regex in a string, item in a list, key in a map

```zil
"price: $12".contains(r"\$\d+")
# → true
[1, 2].contains(2)
# → true
```

See also: [find](data.md#find), [starts_with](text.md#starts_with)

### find

`find(v: str|list, x: any)`: index of the first match, or nil

```zil
"hello".find("l")
# → 2
[5, 6].find(6)
# → 1
"abc".find("z")
# → nil
```

See also: [contains](data.md#contains), [count](data.md#count)

### count

`count(v: str|list, x: any)`: number of matches in a string or list

```zil
"banana".count("a")
# → 3
[1, 2, 1].count(1)
# → 2
```

See also: [find](data.md#find)

### reverse

`reverse(v: str|list)`: reverse a string or list

```zil
"abc".reverse
# → "cba"
[1, 2, 3].reverse
# → [3, 2, 1]
```

See also: [sort](data.md#sort)

### sort

`sort(xs: list, key?: fn)`: sorted copy, optionally by key function

```zil
[3, 1, 2].sort
# → [1, 2, 3]
["ccc", "a", "bb"].sort(|w| w.len)
# → ["a", "bb", "ccc"]
```

See also: [reverse](data.md#reverse), [unique](data.md#unique)

### unique

`unique(xs: list)`: drop duplicates, keeping first occurrences

```zil
[1, 2, 1, 3].unique
# → [1, 2, 3]
```

See also: [sort](data.md#sort), [count](data.md#count)

### first

`first(xs: list)`: first item, or nil

```zil
[7, 8].first
# → 7
```

See also: [last](data.md#last)

### last

`last(xs: list)`: last item, or nil

```zil
[7, 8].last
# → 8
```

See also: [first](data.md#first)

## More examples

### data

```zil
# unique, order kept
[3, 1, 3, 2, 1].unique
# → [3, 1, 2]
# sort quantities
[1 km, 900 m, 1 mi].sort
# → [900 m, 1 km, 1 mi]
# sort dates
[date("2026-12-25"), date("2026-01-01")].sort
# → [2026-01-01, 2026-12-25]
```
