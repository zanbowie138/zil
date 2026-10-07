# math.stats

sums, averages, spread and extremes of lists; units welcome

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`sum(xs: list)`](#sum) | add up a list; works with units |
| [`avg(xs: list)`](#avg) | mean of a list |
| [`product(xs: list)`](#product) | multiply a list together |
| [`median(xs: list)`](#median) | middle value; mean of the middle two for an even count |
| [`mode(xs: list)`](#mode) | most common item; the first one on ties |
| [`percentile(xs: list, p: num)`](#percentile) | the p-th percentile (0-100), interpolating between items |
| [`variance(xs: list)`](#variance) | sample variance (n - 1) |
| [`stdev(xs: list)`](#stdev) | sample standard deviation (n - 1); works with units |
| [`min(xs: list) / min(a: any, b: any, ...)`](#min) | smallest value |
| [`max(xs: list) / max(a: any, b: any, ...)`](#max) | largest value |

### sum

`sum(xs: list)`: add up a list; works with units

```zil
[1, 2, 3].sum
# → 6
[1 m, 50 cm].sum
# → 1.5 m
```

See also: [avg](../math/stats.md#avg), [reduce](../data/lists.md#reduce)

### avg

`avg(xs: list)`: mean of a list

```zil
[1, 2, 4].avg
# → 2.33333
[2 h, 30 min].avg
# → 1.25 h
```

See also: [sum](../math/stats.md#sum), [median](../math/stats.md#median)

### product

`product(xs: list)`: multiply a list together

```zil
[2, 3, 4].product
# → 24
[2 m, 3 m].product
# → 6 m^2
```

See also: [sum](../math/stats.md#sum), [factorial](../math/numtheory.md#factorial)

### median

`median(xs: list)`: middle value; mean of the middle two for an even count

```zil
[3, 1, 2].median
# → 2
[1 m, 3 m, 50 cm, 2 m].median
# → 1.5 m
```

See also: [avg](../math/stats.md#avg), [percentile](../math/stats.md#percentile)

### mode

`mode(xs: list)`: most common item; the first one on ties

```zil
[1, 2, 2, 3].mode
# → 2
"hello".chars.mode
# → "l"
```

See also: [median](../math/stats.md#median), [count](../data.md#count)

### percentile

`percentile(xs: list, p: num)`: the p-th percentile (0-100), interpolating between items

```zil
[1, 2, 3, 4, 5].percentile(90)
# → 4.6
(1..=100).percentile(25)
# → 25.75
```

See also: [median](../math/stats.md#median)

### variance

`variance(xs: list)`: sample variance (n - 1)

```zil
[2, 4, 4, 4, 5, 5, 7, 9].variance
# → 4.57143
```

See also: [stdev](../math/stats.md#stdev)

### stdev

`stdev(xs: list)`: sample standard deviation (n - 1); works with units

```zil
[2, 4, 4, 4, 5, 5, 7, 9].stdev
# → 2.13809
[1 m, 2 m, 3 m].stdev
# → 1 m
```

See also: [variance](../math/stats.md#variance), [avg](../math/stats.md#avg)

### min

`min(xs: list) / min(a: any, b: any, ...)`: smallest value

```zil
min(3, 9, 4)
# → 3
[2 km, 1 mi].min
# → 1 mi
```

See also: [max](../math/stats.md#max), [sort](../data.md#sort)

### max

`max(xs: list) / max(a: any, b: any, ...)`: largest value

```zil
max(3, 9, 4)
# → 9
["b", "a"].max
# → "b"
```

See also: [min](../math/stats.md#min), [sort](../data.md#sort)

## More examples

### stats

```zil
# standard deviation
[2, 4, 4, 4, 5, 5, 7, 9].stdev
# → 2.13809
# percentile
(1..=100).percentile(90)
# → 90.1
```
