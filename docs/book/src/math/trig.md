# math.trig

sine, cosine and friends; angles as units

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`sin(x)`](#sin) | sine of radians or an angle unit |
| [`cos(x)`](#cos) | cosine of radians or an angle unit |
| [`tan(x)`](#tan) | tangent of radians or an angle unit |
| [`asin(x)`](#asin) | arcsine, as an angle |
| [`acos(x)`](#acos) | arccosine, as an angle |
| [`atan(x)`](#atan) | arctangent, as an angle |
| [`atan2(y, x)`](#atan2) | angle of the point (x, y), as an angle |
| [`sinh(x)`](#sinh) | hyperbolic sine |
| [`cosh(x)`](#cosh) | hyperbolic cosine |
| [`tanh(x)`](#tanh) | hyperbolic tangent |

### sin

`sin(x)`: sine of radians or an angle unit

```zil
sin(30 deg)
# → 0.5
sin(pi / 2)
# → 1
```

See also: [cos](../math/trig.md#cos), [tan](../math/trig.md#tan), [asin](../math/trig.md#asin)

### cos

`cos(x)`: cosine of radians or an angle unit

```zil
cos(60 deg)
# → 0.5
```

See also: [sin](../math/trig.md#sin), [tan](../math/trig.md#tan), [acos](../math/trig.md#acos)

### tan

`tan(x)`: tangent of radians or an angle unit

```zil
tan(45 deg)
# → 1
```

See also: [sin](../math/trig.md#sin), [cos](../math/trig.md#cos), [atan](../math/trig.md#atan)

### asin

`asin(x)`: arcsine, as an angle

```zil
asin(1) to deg
# → 90 deg
```

See also: [sin](../math/trig.md#sin)

### acos

`acos(x)`: arccosine, as an angle

```zil
acos(0) to deg
# → 90 deg
```

See also: [cos](../math/trig.md#cos)

### atan

`atan(x)`: arctangent, as an angle

```zil
atan(1) to deg
# → 45 deg
```

See also: [tan](../math/trig.md#tan), [atan2](../math/trig.md#atan2)

### atan2

`atan2(y, x)`: angle of the point (x, y), as an angle

```zil
atan2(1, -1) to deg
# → 135 deg
```

See also: [atan](../math/trig.md#atan), [hypot](../math.md#hypot)

### sinh

`sinh(x)`: hyperbolic sine

```zil
sinh(1)
# → 1.1752
```

See also: [cosh](../math/trig.md#cosh), [tanh](../math/trig.md#tanh)

### cosh

`cosh(x)`: hyperbolic cosine

```zil
cosh(1)
# → 1.54308
```

See also: [sinh](../math/trig.md#sinh), [tanh](../math/trig.md#tanh)

### tanh

`tanh(x)`: hyperbolic tangent

```zil
tanh(1)
# → 0.761594
```

See also: [sinh](../math/trig.md#sinh), [cosh](../math/trig.md#cosh)

## More examples

### trig

```zil
# trig takes angle units
sin(30 deg)
# → 0.5
# radians back to degrees
atan2(1, 1) to deg
# → 45 deg
```
