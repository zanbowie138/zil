# data.lists

ranges, map/filter/reduce, building lists

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### operators

```zil
# a..b
1..4
# → [1, 2, 3]
# a..=b
1..=4
# → [1, 2, 3, 4]
# list + list
[1] + [2, 3]
# → [1, 2, 3]
```

## Functions

| function | description |
|---|---|
| [`range(n: int) / range(a: int, b: int)`](#range) | integers in [0, n) or [a, b); same as a..b |
| [`push(xs: list, v: any)`](#push) | append v in place and return the list |
| [`pop(xs: list)`](#pop) | remove and return the last item in place, or nil if empty |
| [`shift(xs: list)`](#shift) | remove and return the first item in place, or nil if empty |
| [`unshift(xs: list, v: any)`](#unshift) | insert v at the front in place and return the list |
| [`map(xs: list, f: fn)`](#map) | apply f to every item |
| [`filter(xs: list, f: fn) / filter(m: map, f: fn)`](#filter) | keep items where f is truthy; for maps, f gets (key, value) |
| [`reduce(xs: list, init: any, f: fn)`](#reduce) | fold with f(acc, item) |
| [`flatten(xs: list)`](#flatten) | unpack nested lists one level |
| [`zip(a: list, b: list)`](#zip) | pair up items, stopping at the shorter list |
| [`enumerate(xs: list)`](#enumerate) | [index, item] pairs |
| [`group_by(xs: list, f: fn)`](#group_by) | map from each key f gives to the items with that key |
| [`count_by(xs: list, f: fn)`](#count_by) | map from each key f gives to how many items have it |
| [`chunks(xs: list, n: int)`](#chunks) | split into lists of n items; the last may be shorter |
| [`windows(xs: list, n: int)`](#windows) | every run of n neighboring items |
| [`step(xs: list, n: int)`](#step) | every nth item, starting with the first |

### range

`range(n: int) / range(a: int, b: int)`: integers in [0, n) or [a, b); same as a..b

```zil
range(4)
# → [0, 1, 2, 3]
range(2, 5)
# → [2, 3, 4]
```

See also: [map](../data/lists.md#map)

### push

`push(xs: list, v: any)`: append v in place and return the list

```zil
[1, 2].push(3)
# → [1, 2, 3]
```

See also: [pop](../data/lists.md#pop), [unshift](../data/lists.md#unshift)

### pop

`pop(xs: list)`: remove and return the last item in place, or nil if empty

```zil
xs = [1, 2, 3]; [xs.pop, xs]
# → [3, [1, 2]]
```

See also: [push](../data/lists.md#push), [shift](../data/lists.md#shift)

### shift

`shift(xs: list)`: remove and return the first item in place, or nil if empty

```zil
xs = [1, 2, 3]; [xs.shift, xs]
# → [1, [2, 3]]
```

See also: [unshift](../data/lists.md#unshift), [pop](../data/lists.md#pop)

### unshift

`unshift(xs: list, v: any)`: insert v at the front in place and return the list

```zil
[2, 3].unshift(1)
# → [1, 2, 3]
```

See also: [shift](../data/lists.md#shift), [push](../data/lists.md#push)

### map

`map(xs: list, f: fn)`: apply f to every item

```zil
[1, 2, 3].map(|x| x * 10)
# → [10, 20, 30]
```

See also: [filter](../data/lists.md#filter), [reduce](../data/lists.md#reduce)

### filter

`filter(xs: list, f: fn) / filter(m: map, f: fn)`: keep items where f is truthy; for maps, f gets (key, value)

```zil
(1..10).filter(|x| x % 3 == 0)
# → [3, 6, 9]
{a: 1, b: 5}.filter(|k, v| v > 2)
# → {b: 5}
```

See also: [map](../data/lists.md#map), [reduce](../data/lists.md#reduce), [filter_keys](../data/maps.md#filter_keys)

### reduce

`reduce(xs: list, init: any, f: fn)`: fold with f(acc, item)

```zil
[1, 2, 3].reduce(10, |acc, x| acc + x)
# → 16
```

See also: [sum](../math/stats.md#sum), [map](../data/lists.md#map)

### flatten

`flatten(xs: list)`: unpack nested lists one level

```zil
[[1, 2], [3], 4].flatten
# → [1, 2, 3, 4]
```

See also: [chunks](../data/lists.md#chunks)

### zip

`zip(a: list, b: list)`: pair up items, stopping at the shorter list

```zil
zip([1, 2, 3], ["a", "b"])
# → [[1, "a"], [2, "b"]]
```

See also: [enumerate](../data/lists.md#enumerate)

### enumerate

`enumerate(xs: list)`: [index, item] pairs

```zil
["a", "b"].enumerate
# → [[0, "a"], [1, "b"]]
["a", "b"].enumerate.map(|[i, x]| "{i}:{x}")
# → ["0:a", "1:b"]
```

See also: [zip](../data/lists.md#zip)

### group_by

`group_by(xs: list, f: fn)`: map from each key f gives to the items with that key

```zil
["apple", "avocado", "banana"].group_by(|w| w[0])
# → {a: ["apple", "avocado"], b: ["banana"]}
(1..=6).group_by(|x| x % 2 == 0)
# → {false: [1, 3, 5], true: [2, 4, 6]}
```

See also: [count_by](../data/lists.md#count_by)

### count_by

`count_by(xs: list, f: fn)`: map from each key f gives to how many items have it

```zil
"mississippi".chars.count_by(|c| c)
# → {m: 1, i: 4, s: 4, p: 2}
```

See also: [group_by](../data/lists.md#group_by), [count](../data.md#count)

### chunks

`chunks(xs: list, n: int)`: split into lists of n items; the last may be shorter

```zil
(1..=7).chunks(3)
# → [[1, 2, 3], [4, 5, 6], [7]]
```

See also: [windows](../data/lists.md#windows), [flatten](../data/lists.md#flatten)

### windows

`windows(xs: list, n: int)`: every run of n neighboring items

```zil
[1, 2, 3, 4].windows(2)
# → [[1, 2], [2, 3], [3, 4]]
[1, 4, 9, 16].windows(2).map(|[a, b]| b - a)
# → [3, 5, 7]
```

See also: [chunks](../data/lists.md#chunks)

### step

`step(xs: list, n: int)`: every nth item, starting with the first

```zil
(0..=20).step(5)
# → [0, 5, 10, 15, 20]
(1..10).step(2)
# → [1, 3, 5, 7, 9]
```

See also: [range](../data/lists.md#range)

## More examples

### lists

```zil
# stepped range
(1..=10).step(3)
# → [1, 4, 7, 10]
# primes under 100
(1..100).filter(is_prime).len
# → 25
# pipe into a function
(1..=10).map(|x| x ** 2) |> sum
# → 385
# fold
[1, 2, 3].reduce(10, |acc, x| acc + x)
# → 16
```
