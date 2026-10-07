# math.calculus

numeric roots, derivatives and integrals of functions

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`root(f: fn, guess: num) / root(f: fn, lo: num, hi: num)`](#root) | an x where f(x) = 0: Newton's method from a guess, or bisection between lo and hi where f changes sign |
| [`deriv(f: fn, x: num)`](#deriv) | f'(x) by central difference |
| [`integrate(f: fn, a: num\|quantity, b: num\|quantity)`](#integrate) | ∫ f from a to b by Simpson's rule (1000 steps); units carry through |

### root

`root(f: fn, guess: num) / root(f: fn, lo: num, hi: num)`: an x where f(x) = 0: Newton's method from a guess, or bisection between lo and hi where f changes sign

```zil
root(|x| x ** 2 - 2, 1)
# → 1.41421
root(|x| x ** 3 - x - 1, 1, 2)
# → 1.32472
```

See also: [deriv](../math/calculus.md#deriv)

### deriv

`deriv(f: fn, x: num)`: f'(x) by central difference

```zil
deriv(|x| x ** 2, 3)
# → 6
deriv(sin, 0)
# → 1
```

See also: [integrate](../math/calculus.md#integrate), [root](../math/calculus.md#root)

### integrate

`integrate(f: fn, a: num|quantity, b: num|quantity)`: ∫ f from a to b by Simpson's rule (1000 steps); units carry through

```zil
integrate(|x| x ** 2, 0, 3)
# → 9
integrate(sin, 0, pi)
# → 2
```

See also: [deriv](../math/calculus.md#deriv)

## More examples

### calculus

```zil
# √2 the hard way
root(|x| x ** 2 - 2, 1)
# → 1.41421
# where cos meets x
root(|x| cos(x) - x, 0, 1)
# → 0.739085
# slope of x³ at 2
deriv(|x| x ** 3, 2)
# → 12
# area under a bell curve
integrate(|x| exp(-x ** 2), -10, 10) ** 2
# → 3.14159
# distance from speed
integrate(|t| 9.8 m/s^2 * t, 0 s, 3 s)
# → 44.1 m
```
