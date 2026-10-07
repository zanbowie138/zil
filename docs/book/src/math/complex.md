# math.complex

complex numbers, `2 + 3i`; a number touching `i` is imaginary

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### literals

```zil
# ni
3i
# a + bi
2 + 3i
```

### operators

```zil
# + - * / **
(1 + 2i) * (3 - 1i)
# → 5 + 5i
```

### also takes complex

```zil
# abs exp ln sqrt
sqrt(-4 + 0i)
# → 2i
```

## Functions

| function | description |
|---|---|
| [`re(z: num\|complex)`](#re) | real part |
| [`im(z: num\|complex)`](#im) | imaginary part |
| [`conj(z: num\|complex)`](#conj) | complex conjugate |
| [`arg(z: num\|complex)`](#arg) | angle from the positive real axis, as an angle |
| [`csqrt(z: num\|complex)`](#csqrt) | square root that goes complex for negatives (principal root) |
| [`polar(r: num, theta: num\|quantity) / polar(z: num\|complex)`](#polar) | a complex number from a length and an angle; with one argument, z in polar form (`to polar`) |

### re

`re(z: num|complex)`: real part

```zil
re(2 + 3i)
# → 2
```

See also: [im](../math/complex.md#im)

### im

`im(z: num|complex)`: imaginary part

```zil
im(2 + 3i)
# → 3
```

See also: [re](../math/complex.md#re)

### conj

`conj(z: num|complex)`: complex conjugate

```zil
conj(2 + 3i)
# → 2 - 3i
```

See also: [re](../math/complex.md#re), [im](../math/complex.md#im)

### arg

`arg(z: num|complex)`: angle from the positive real axis, as an angle

```zil
arg(1i) to deg
# → 90 deg
```

See also: [polar](../math/complex.md#polar), [abs](../math.md#abs)

### csqrt

`csqrt(z: num|complex)`: square root that goes complex for negatives (principal root)

```zil
csqrt(-4)
# → 2i
```

See also: [sqrt](../math.md#sqrt)

### polar

`polar(r: num, theta: num|quantity) / polar(z: num|complex)`: a complex number from a length and an angle; with one argument, z in polar form (`to polar`)

```zil
polar(2, 90 deg)
# → 2i
polar(1i)
# → "polar(1, 90 deg)"
```

See also: [arg](../math/complex.md#arg), [abs](../math.md#abs)

## More examples

### complex

```zil
# Euler's identity
e ** (1i * pi)
# → -1 + 0i
# roots of x² + 2x + 5
[-1 + csqrt(-4) / 2, -1 - csqrt(-4) / 2]
# → [-1 + 1i, -1 - 1i]
# polar form
1 + 1i to polar
# → "polar(1.41421, 45 deg)"
```
