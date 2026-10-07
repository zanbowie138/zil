# math

rounding, roots, logs, constants; exact fractions and big ints

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

### constants

```zil
# pi e tau phi inf nan
```

### operators

```zil
# n%
20%
# → 0.2
# x ± n%
50 + 10%
# → 55
# n% of x
20% of 50
# → 10
```

## Submodules

| module | about |
|---|---|
| [stats](math/stats.md) | sums, averages, spread and extremes of lists; units welcome |
| [trig](math/trig.md) | sine, cosine and friends; angles as units |
| [numtheory](math/numtheory.md) | gcd, primes, factoring, factorials, modular powers; exact at any size |
| [bits](math/bits.md) | count, test, set, rotate and swap bits |
| [bases](math/bases.md) | hex, binary, octal and any base 2-36; digits of a number |
| [formatting](math/formatting.md) | format specs in strings, printf, and fixed/sci/percent/commas |
| [random](math/random.md) | numbers, picks, shuffles, UUIDs |

## Functions

| function | description |
|---|---|
| [`sqrt(x)`](#sqrt) | square root |
| [`abs(x)`](#abs) | absolute value; keeps units |
| [`round(x, digits?) / round(x, step)`](#round) | round to nearest; keeps units. A non-int second arg rounds to a multiple of it |
| [`floor(x)`](#floor) | round down |
| [`ceil(x)`](#ceil) | round up |
| [`trunc(x)`](#trunc) | round toward zero; keeps units |
| [`sign(x)`](#sign) | -1, 0 or 1 |
| [`clamp(x, lo, hi)`](#clamp) | limit x to [lo, hi] |
| [`cbrt(x)`](#cbrt) | cube root |
| [`exp(x)`](#exp) | e to the power x |
| [`ln(x)`](#ln) | natural log |
| [`log(x, base?)`](#log) | log base 10, or another base |
| [`hypot(x, y)`](#hypot) | sqrt(x² + y²) without overflow; keeps units |
| [`is_nan(x)`](#is_nan) | whether x is nan (nan != nan) |

### sqrt

`sqrt(x)`: square root

```zil
sqrt(2)
# → 1.41421
```

See also: [cbrt](math.md#cbrt)

### abs

`abs(x)`: absolute value; keeps units

```zil
abs(-3)
# → 3
abs(-2 km)
# → 2 km
```

See also: [round](math.md#round), [sign](math.md#sign)

### round

`round(x, digits?) / round(x, step)`: round to nearest; keeps units. A non-int second arg rounds to a multiple of it

```zil
round(pi, 2)
# → 3.14
round(2.5 km)
# → 3 km
round(7.3, 0.25)
# → 7.25
round(17 min, 15 min)
# → 15 min
```

See also: [floor](math.md#floor), [ceil](math.md#ceil), [trunc](math.md#trunc)

### floor

`floor(x)`: round down

```zil
floor(2.7)
# → 2
```

See also: [ceil](math.md#ceil), [round](math.md#round)

### ceil

`ceil(x)`: round up

```zil
ceil(2.1)
# → 3
```

See also: [floor](math.md#floor), [round](math.md#round)

### trunc

`trunc(x)`: round toward zero; keeps units

```zil
trunc(-2.7)
# → -2
```

See also: [floor](math.md#floor), [round](math.md#round)

### sign

`sign(x)`: -1, 0 or 1

```zil
sign(-5 km)
# → -1
sign(0)
# → 0
```

See also: [abs](math.md#abs)

### clamp

`clamp(x, lo, hi)`: limit x to [lo, hi]

```zil
clamp(15, 0, 10)
# → 10
clamp(5 m, 1 m, 2 m)
# → 2 m
```

See also: [min](math/stats.md#min), [max](math/stats.md#max)

### cbrt

`cbrt(x)`: cube root

```zil
cbrt(27)
# → 3
```

See also: [sqrt](math.md#sqrt)

### exp

`exp(x)`: e to the power x

```zil
exp(1)
# → 2.71828
```

See also: [ln](math.md#ln)

### ln

`ln(x)`: natural log

```zil
ln(e)
# → 1
```

See also: [log](math.md#log), [exp](math.md#exp)

### log

`log(x, base?)`: log base 10, or another base

```zil
log(1000)
# → 3
log(8, 2)
# → 3
```

See also: [ln](math.md#ln)

### hypot

`hypot(x, y)`: sqrt(x² + y²) without overflow; keeps units

```zil
hypot(3, 4)
# → 5
hypot(3 m, 4 m)
# → 5 m
```

See also: [sqrt](math.md#sqrt), [atan2](math/trig.md#atan2)

### is_nan

`is_nan(x)`: whether x is nan (nan != nan)

```zil
is_nan(nan)
# → true
is_nan(inf - inf)
# → true
```

## More examples

### math

```zil
# exact fractions
(1..=6).map(|x| 1/x).sum to frac
# → 49/20
# decimals are exact
0.1 + 0.2 == 0.3
# → true
# 1/3 * 3 is exactly 1
1/3 * 3 == 1
# → true
# decimal to fraction
0.75 to frac
# → 3/4
# big ints never overflow
2 ** 100
# → 1267650600228229401496703205376
# add a percent
80 + 15%
# → 92
# percent of
15% of 64.50
# → 9.675
```
