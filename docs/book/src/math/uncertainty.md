# math.uncertainty

measurements with an error, `5 ± 0.1 m`, propagated through arithmetic and math (first order, independent errors)

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### operators

```zil
# x ± e (or x +- e)
5 ± 0.1 m
# x ± n%
20 ± 5%
# → 20 ± 1
```

### propagation

```zil
# errors add in quadrature
(5 ± 0.3) + (2 ± 0.4)
# → 7 ± 0.5
# relative errors too
(10 ± 1 m) * (2 ± 0.1 m)
# → 20 ± 2.23607 m^2
# through functions
sqrt(16 ± 1)
# → 4 ± 0.125
# and conversions
5 ± 0.1 km to mi
# → 3.10686 ± 0.0621371 mi
```

## Functions

| function | description |
|---|---|
| [`value(x: uncertain)`](#value) | the central value, dropping the uncertainty |
| [`error(x: uncertain)`](#error) | the uncertainty, in the same unit |

### value

`value(x: uncertain)`: the central value, dropping the uncertainty

```zil
(5 ± 0.1 m).value
# → 5 m
```

See also: [error](../math/uncertainty.md#error)

### error

`error(x: uncertain)`: the uncertainty, in the same unit

```zil
(5 ± 0.1 m).error
# → 0.1 m
(20 ± 5%).error
# → 1
```

See also: [value](../math/uncertainty.md#value)

## More examples

### uncertainty

```zil
# area of a measured rectangle
(2 ± 0.05 m) * (3 ± 0.05 m)
# → 6 ± 0.180278 m^2
# pendulum period from its length in m
2 * pi * sqrt((1 ± 0.01) / 9.81)
# → 2.00607 ± 0.0100303
```
