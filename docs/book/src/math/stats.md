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
| [`describe(xs: list)`](#describe) | n, mean, stdev, min, quartiles and max in one map; units welcome |
| [`corr(xs: list, ys: list)`](#corr) | Pearson correlation, from -1 to 1 |
| [`fit(xs: list, ys: list)`](#fit) | least-squares line: {slope, intercept, r2} |
| [`zscore(xs: list)`](#zscore) | how many standard deviations each item is from the mean |
| [`normalize(xs: list)`](#normalize) | rescale so the min is 0 and the max is 1 |
| [`cumsum(xs: list)`](#cumsum) | running totals |
| [`deltas(xs: list)`](#deltas) | the difference between each item and the one before |
| [`lerp(a: any, b: any, t: num)`](#lerp) | a + (b - a) * t: the point a fraction t of the way from a to b; numbers, units or dates |
| [`remap(x: any, lo: any, hi: any, to_lo: any, to_hi: any)`](#remap) | x moved from the range lo..hi to the range to_lo..to_hi, proportionally |

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

### describe

`describe(xs: list)`: n, mean, stdev, min, quartiles and max in one map; units welcome

```zil
[3, 1, 4, 1, 5, 9, 2, 6].describe
# → {n: 8, mean: 3.875, stdev: 2.74838, min: 1, p25: 1.75, median: 3.5, p75: 5.25, max: 9}
```

See also: [avg](../math/stats.md#avg), [percentile](../math/stats.md#percentile)

### corr

`corr(xs: list, ys: list)`: Pearson correlation, from -1 to 1

```zil
corr([1, 2, 3, 4], [2, 4, 5, 9])
# → 0.964764
```

See also: [fit](../math/stats.md#fit)

### fit

`fit(xs: list, ys: list)`: least-squares line: {slope, intercept, r2}

```zil
fit([1, 2, 3], [2, 4, 6])
# → {slope: 2, intercept: 0, r2: 1}
fit([1, 2, 3, 4], [2.1, 3.9, 6.2, 7.8])
# → {slope: 1.94, intercept: 0.15, r2: 0.995661}
```

See also: [corr](../math/stats.md#corr)

### zscore

`zscore(xs: list)`: how many standard deviations each item is from the mean

```zil
[2, 4, 4, 4, 5, 5, 7, 9].zscore
# → [-1.40312, -0.467707, -0.467707, -0.467707, 0, 0, 0.935414, 1.87083]
```

See also: [normalize](../math/stats.md#normalize), [stdev](../math/stats.md#stdev)

### normalize

`normalize(xs: list)`: rescale so the min is 0 and the max is 1

```zil
[10, 15, 20].normalize
# → [0, 0.5, 1]
```

See also: [zscore](../math/stats.md#zscore), [remap](../math/stats.md#remap)

### cumsum

`cumsum(xs: list)`: running totals

```zil
[1, 2, 3, 4].cumsum
# → [1, 3, 6, 10]
[1 km, 500 m].cumsum
# → [1 km, 1.5 km]
```

See also: [deltas](../math/stats.md#deltas), [sum](../math/stats.md#sum)

### deltas

`deltas(xs: list)`: the difference between each item and the one before

```zil
[1, 4, 9, 16].deltas
# → [3, 5, 7]
[1, 4, 9, 16].deltas.deltas
# → [2, 2]
```

See also: [cumsum](../math/stats.md#cumsum), [windows](../data/lists.md#windows)

### lerp

`lerp(a: any, b: any, t: num)`: a + (b - a) * t: the point a fraction t of the way from a to b; numbers, units or dates

```zil
lerp(10, 20, 0.25)
# → 12.5
lerp(0 C, 100 C, 0.37)
# → 37 C
```

See also: [remap](../math/stats.md#remap)

### remap

`remap(x: any, lo: any, hi: any, to_lo: any, to_hi: any)`: x moved from the range lo..hi to the range to_lo..to_hi, proportionally

```zil
remap(5, 0, 10, 100, 200)
# → 150
remap(72 F, 32 F, 212 F, 0, 100)
# → 22.2222
```

See also: [lerp](../math/stats.md#lerp), [normalize](../math/stats.md#normalize)

## More examples

### stats

```zil
# standard deviation
[2, 4, 4, 4, 5, 5, 7, 9].stdev
# → 2.13809
# percentile
(1..=100).percentile(90)
# → 90.1
# summary
[3, 1, 4, 1, 5, 9, 2, 6].describe
# → {n: 8, mean: 3.875, stdev: 2.74838, min: 1, p25: 1.75, median: 3.5, p75: 5.25, max: 9}
# trend line
fit([1, 2, 3, 4], [2.1, 3.9, 6.2, 7.8])
# → {slope: 1.94, intercept: 0.15, r2: 0.995661}
# running total
[$5, $12, $3].cumsum
# → [$5.00, $17.00, $20.00]
# halfway between dates
lerp(date("2026-01-01"), date("2026-12-31"), 1/2)
# → 2026-07-02
# °C to a 0-255 byte
remap(25, 0, 40, 0, 255).round
# → 159
```
