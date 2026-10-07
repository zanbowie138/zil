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
| [`map(xs: list, f: fn)`](#map) | apply f to every item |
| [`filter(xs: list, f: fn)`](#filter) | keep items where f is truthy |
| [`reduce(xs: list, init: any, f: fn)`](#reduce) | fold with f(acc, item) |
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

### map

`map(xs: list, f: fn)`: apply f to every item

```zil
[1, 2, 3].map(|x| x * 10)
# → [10, 20, 30]
```

See also: [filter](../data/lists.md#filter), [reduce](../data/lists.md#reduce)

### filter

`filter(xs: list, f: fn)`: keep items where f is truthy

```zil
(1..10).filter(|x| x % 3 == 0)
# → [3, 6, 9]
```

See also: [map](../data/lists.md#map), [reduce](../data/lists.md#reduce)

### reduce

`reduce(xs: list, init: any, f: fn)`: fold with f(acc, item)

```zil
[1, 2, 3].reduce(10, |acc, x| acc + x)
# → 16
```

See also: [sum](../math/stats.md#sum), [map](../data/lists.md#map)

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
