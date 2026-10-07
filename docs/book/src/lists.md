# lists

ranges, higher-order fns, aggregates, sorting

> Example results generated on 2026-10-06.
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
# a..b
1..4
# → [1, 2, 3]
# a..=b
1..=4
# → [1, 2, 3, 4]
# x in list
2 in [1, 2]
# → true
# list + list
[1] + [2, 3]
# → [1, 2, 3]
```

## Functions

| function | description |
|---|---|
| [`range(n) / range(a, b)`](#range) | integers in [0, n) or [a, b); same as a..b |
| [`push(list, v)`](#push) | append v in place and return the list |
| [`map(list, f)`](#map) | apply f to every item |
| [`filter(list, f)`](#filter) | keep items where f is truthy |
| [`reduce(list, init, f)`](#reduce) | fold with f(acc, item) |
| [`sum(list)`](#sum) | add up a list; works with units |
| [`avg(list)`](#avg) | mean of a list |
| [`product(list)`](#product) | multiply a list together |
| [`median(list)`](#median) | middle value; mean of the middle two for an even count |
| [`mode(list)`](#mode) | most common item; the first one on ties |
| [`percentile(list, p)`](#percentile) | the p-th percentile (0-100), interpolating between items |
| [`variance(list)`](#variance) | sample variance (n - 1) |
| [`stdev(list)`](#stdev) | sample standard deviation (n - 1); works with units |
| [`min(list) / min(a, b, ...)`](#min) | smallest value |
| [`max(list) / max(a, b, ...)`](#max) | largest value |
| [`sort(list, key?)`](#sort) | sorted copy, optionally by key function |
| [`unique(list)`](#unique) | drop duplicates, keeping first occurrences |
| [`first(list)`](#first) | first item, or nil |
| [`last(list)`](#last) | last item, or nil |
| [`step(list, n)`](#step) | every nth item, starting with the first |
| [`keys(map)`](#keys) | list of map keys |
| [`values(map)`](#values) | list of map values |

### range

`range(n) / range(a, b)`: integers in [0, n) or [a, b); same as a..b

```zil
range(4)
# → [0, 1, 2, 3]
range(2, 5)
# → [2, 3, 4]
```

See also: [map](lists.md#map)

### push

`push(list, v)`: append v in place and return the list

```zil
[1, 2].push(3)
# → [1, 2, 3]
```

### map

`map(list, f)`: apply f to every item

```zil
[1, 2, 3].map(|x| x * 10)
# → [10, 20, 30]
```

See also: [filter](lists.md#filter), [reduce](lists.md#reduce)

### filter

`filter(list, f)`: keep items where f is truthy

```zil
(1..10).filter(|x| x % 3 == 0)
# → [3, 6, 9]
```

See also: [map](lists.md#map), [reduce](lists.md#reduce)

### reduce

`reduce(list, init, f)`: fold with f(acc, item)

```zil
[1, 2, 3].reduce(10, |acc, x| acc + x)
# → 16
```

See also: [sum](lists.md#sum), [map](lists.md#map)

### sum

`sum(list)`: add up a list; works with units

```zil
[1, 2, 3].sum
# → 6
[1 m, 50 cm].sum
# → 1.5 m
```

See also: [avg](lists.md#avg), [reduce](lists.md#reduce)

### avg

`avg(list)`: mean of a list

```zil
[1, 2, 4].avg
# → 2.33333
[2 h, 30 min].avg
# → 1.25 h
```

See also: [sum](lists.md#sum), [median](lists.md#median)

### product

`product(list)`: multiply a list together

```zil
[2, 3, 4].product
# → 24
[2 m, 3 m].product
# → 6 m^2
```

See also: [sum](lists.md#sum), [factorial](math.md#factorial)

### median

`median(list)`: middle value; mean of the middle two for an even count

```zil
[3, 1, 2].median
# → 2
[1 m, 3 m, 50 cm, 2 m].median
# → 1.5 m
```

See also: [avg](lists.md#avg), [percentile](lists.md#percentile)

### mode

`mode(list)`: most common item; the first one on ties

```zil
[1, 2, 2, 3].mode
# → 2
"hello".chars.mode
# → "l"
```

See also: [median](lists.md#median), [count](strings.md#count)

### percentile

`percentile(list, p)`: the p-th percentile (0-100), interpolating between items

```zil
[1, 2, 3, 4, 5].percentile(90)
# → 4.6
(1..=100).percentile(25)
# → 25.75
```

See also: [median](lists.md#median)

### variance

`variance(list)`: sample variance (n - 1)

```zil
[2, 4, 4, 4, 5, 5, 7, 9].variance
# → 4.57143
```

See also: [stdev](lists.md#stdev)

### stdev

`stdev(list)`: sample standard deviation (n - 1); works with units

```zil
[2, 4, 4, 4, 5, 5, 7, 9].stdev
# → 2.13809
[1 m, 2 m, 3 m].stdev
# → 1 m
```

See also: [variance](lists.md#variance), [avg](lists.md#avg)

### min

`min(list) / min(a, b, ...)`: smallest value

```zil
min(3, 9, 4)
# → 3
[2 km, 1 mi].min
# → 1 mi
```

See also: [max](lists.md#max), [sort](lists.md#sort)

### max

`max(list) / max(a, b, ...)`: largest value

```zil
max(3, 9, 4)
# → 9
["b", "a"].max
# → "b"
```

See also: [min](lists.md#min), [sort](lists.md#sort)

### sort

`sort(list, key?)`: sorted copy, optionally by key function

```zil
[3, 1, 2].sort
# → [1, 2, 3]
["ccc", "a", "bb"].sort(|w| w.len)
# → ["a", "bb", "ccc"]
```

See also: [reverse](strings.md#reverse), [unique](lists.md#unique)

### unique

`unique(list)`: drop duplicates, keeping first occurrences

```zil
[1, 2, 1, 3].unique
# → [1, 2, 3]
```

See also: [sort](lists.md#sort), [count](strings.md#count)

### first

`first(list)`: first item, or nil

```zil
[7, 8].first
# → 7
```

See also: [last](lists.md#last)

### last

`last(list)`: last item, or nil

```zil
[7, 8].last
# → 8
```

See also: [first](lists.md#first)

### step

`step(list, n)`: every nth item, starting with the first

```zil
(0..=20).step(5)
# → [0, 5, 10, 15, 20]
(1..10).step(2)
# → [1, 3, 5, 7, 9]
```

See also: [range](lists.md#range)

### keys

`keys(map)`: list of map keys

```zil
{a: 1, b: 2}.keys
# → ["a", "b"]
```

See also: [values](lists.md#values)

### values

`values(map)`: list of map values

```zil
{a: 1, b: 2}.values
# → [1, 2]
```

See also: [keys](lists.md#keys)

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
# standard deviation
[2, 4, 4, 4, 5, 5, 7, 9].stdev
# → 2.13809
# percentile
(1..=100).percentile(90)
# → 90.1
# unique, order kept
[3, 1, 3, 2, 1].unique
# → [3, 1, 2]
# sort quantities
[1 km, 900 m, 1 mi].sort
# → [900 m, 1 km, 1 mi]
# sort dates
[date("2026-12-25"), date("2026-01-01")].sort
# → [2026-01-01, 2026-12-25]
# sum a map
{a: 1, b: 2}.values.sum
# → 3
```
