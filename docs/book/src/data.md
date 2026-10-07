# data

length, search, sorting and picking across strings, lists, maps and sets; lists, maps and sets below

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
| [maps](data/maps.md) | keys, values, lookups, merging and transforming maps |
| [sets](data/sets.md) | unique values in insertion order: union, intersection, difference |

## Functions

| function | description |
|---|---|
| [`len(v: str\|list\|map\|set)`](#len) | length of a string, list, map or set |
| [`contains(v: str\|list\|map\|set, x: any)`](#contains) | substring/regex in a string, item in a list or set, key in a map |
| [`find(v: str\|list, x: any)`](#find) | index of the first match, or nil |
| [`count(v: str\|list, x: any)`](#count) | number of matches in a string or list |
| [`reverse(v: str\|list)`](#reverse) | reverse a string or list |
| [`sort(xs: list\|set, key?: fn)`](#sort) | sorted list, optionally by key function |
| [`sort_desc(xs: list\|set, key?: fn)`](#sort_desc) | like sort, largest first |
| [`unique(xs: list)`](#unique) | drop duplicates, keeping first occurrences |
| [`first(xs: list)`](#first) | first item, or nil |
| [`last(xs: list)`](#last) | last item, or nil |
| [`take(v: str\|list, n: int)`](#take) | the first n items or characters |
| [`drop(v: str\|list, n: int)`](#drop) | everything after the first n items or characters |
| [`any(xs: list, f?: fn)`](#any) | true if f (or the item itself) is truthy for some item |
| [`all(xs: list, f?: fn)`](#all) | true if f (or the item itself) is truthy for every item |

### len

`len(v: str|list|map|set)`: length of a string, list, map or set

```zil
"héllo".len
# → 5
[1, 2, 3].len
# → 3
{a: 1}.len
# → 1
```

### contains

`contains(v: str|list|map|set, x: any)`: substring/regex in a string, item in a list or set, key in a map

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

`sort(xs: list|set, key?: fn)`: sorted list, optionally by key function

```zil
[3, 1, 2].sort
# → [1, 2, 3]
["ccc", "a", "bb"].sort(|w| w.len)
# → ["a", "bb", "ccc"]
```

See also: [reverse](data.md#reverse), [unique](data.md#unique)

### sort_desc

`sort_desc(xs: list|set, key?: fn)`: like sort, largest first

```zil
[3, 1, 2].sort_desc
# → [3, 2, 1]
["bb", "a", "ccc"].sort_desc(|w| w.len)
# → ["ccc", "bb", "a"]
```

See also: [sort](data.md#sort)

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

### take

`take(v: str|list, n: int)`: the first n items or characters

```zil
[1, 2, 3].take(2)
# → [1, 2]
"hello".take(3)
# → "hel"
```

See also: [drop](data.md#drop), [first](data.md#first)

### drop

`drop(v: str|list, n: int)`: everything after the first n items or characters

```zil
[1, 2, 3].drop(2)
# → [3]
"hello".drop(3)
# → "lo"
```

See also: [take](data.md#take), [last](data.md#last)

### any

`any(xs: list, f?: fn)`: true if f (or the item itself) is truthy for some item

```zil
[0, 5, 12].any(|x| x > 10)
# → true
[nil, false].any
# → false
```

See also: [all](data.md#all), [filter](data/lists.md#filter)

### all

`all(xs: list, f?: fn)`: true if f (or the item itself) is truthy for every item

```zil
[2, 4, 6].all(|x| x % 2 == 0)
# → true
[].all
# → true
```

See also: [any](data.md#any), [filter](data/lists.md#filter)

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
