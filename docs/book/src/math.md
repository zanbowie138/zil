# math

digits, rounding, logs, trigonometry (angles as units), formatting, number theory, bits

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

### format specs

```zil
# "{x:.2f}"  fixed decimals
"{pi:.2f}"
# → "3.14"
# "{x:>8}"  width, align < > ^
"[{42:>8}]"
# → "[      42]"
# "{n:08x}"  zero pad; x X b o d
"{255:08x}"
# → "000000ff"
# "{n:#06x}"  # adds 0x 0b 0o
"{255:#06x}"
# → "0x00ff"
# "{x:,}"  commas; also + e %
"{1234567:,}"
# → "1,234,567"
# format("%5.2f", x)  printf
format("%5.2f|%-4d|", pi, 7)
# → " 3.14|7   |"
```

### conversions

```zil
# to bits
0xf0 to bits
# → "0b1111_0000"
```

## Functions

| function | description |
|---|---|
| [`digits(n)`](#digits) | list of digits in the number's own base |
| [`from_digits(list, base?)`](#from_digits) | build a number from digits, kept in that base |
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
| [`fixed(x, digits)`](#fixed) | string with exactly that many decimals; keeps units |
| [`sci(x, digits?)`](#sci) | string in scientific notation |
| [`percent(x, digits?)`](#percent) | string as a percentage |
| [`commas(x, digits?)`](#commas) | string with thousands separators |
| [`gcd(a, b, ...) / gcd(list)`](#gcd) | greatest common divisor |
| [`lcm(a, b, ...) / lcm(list)`](#lcm) | least common multiple |
| [`is_prime(n)`](#is_prime) | primality (Miller-Rabin; exact below 3e24) |
| [`factors(n)`](#factors) | prime factors, smallest first |
| [`factorial(n)`](#factorial) | n!, exact |
| [`choose(n, k)`](#choose) | ways to pick k of n, exact |
| [`mod_pow(b, e, m)`](#mod_pow) | b ** e % m without the huge power |
| [`popcount(n)`](#popcount) | number of 1 bits (two's complement for negatives) |
| [`bit(n, i)`](#bit) | bit i of n (0 is the lowest), as 0 or 1 |
| [`set_bit(n, i)`](#set_bit) | n with bit i set |
| [`clear_bit(n, i)`](#clear_bit) | n with bit i cleared |
| [`rotl(n, k, width)`](#rotl) | rotate the low width bits left by k |
| [`rotr(n, k, width)`](#rotr) | rotate the low width bits right by k |
| [`byteswap(n, width)`](#byteswap) | reverse the bytes of a width-bit number |

### digits

`digits(n)`: list of digits in the number's own base

```zil
1234.digits
# → [1, 2, 3, 4]
0b1011.digits
# → [1, 0, 1, 1]
```

See also: [from_digits](math.md#from_digits)

### from_digits

`from_digits(list, base?)`: build a number from digits, kept in that base

```zil
[1, 2, 3].from_digits
# → 123
[1, 0, 1, 1].from_digits(2)
# → 0b1011
```

See also: [digits](math.md#digits)

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

See also: [min](lists.md#min), [max](lists.md#max)

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

See also: [sqrt](math.md#sqrt), [atan2](math.md#atan2)

### is_nan

`is_nan(x)`: whether x is nan (nan != nan)

```zil
is_nan(nan)
# → true
is_nan(inf - inf)
# → true
```

### sin

`sin(x)`: sine of radians or an angle unit

```zil
sin(30 deg)
# → 0.5
sin(pi / 2)
# → 1
```

See also: [cos](math.md#cos), [tan](math.md#tan), [asin](math.md#asin)

### cos

`cos(x)`: cosine of radians or an angle unit

```zil
cos(60 deg)
# → 0.5
```

See also: [sin](math.md#sin), [tan](math.md#tan), [acos](math.md#acos)

### tan

`tan(x)`: tangent of radians or an angle unit

```zil
tan(45 deg)
# → 1
```

See also: [sin](math.md#sin), [cos](math.md#cos), [atan](math.md#atan)

### asin

`asin(x)`: arcsine, as an angle

```zil
asin(1) to deg
# → 90 deg
```

See also: [sin](math.md#sin)

### acos

`acos(x)`: arccosine, as an angle

```zil
acos(0) to deg
# → 90 deg
```

See also: [cos](math.md#cos)

### atan

`atan(x)`: arctangent, as an angle

```zil
atan(1) to deg
# → 45 deg
```

See also: [tan](math.md#tan), [atan2](math.md#atan2)

### atan2

`atan2(y, x)`: angle of the point (x, y), as an angle

```zil
atan2(1, -1) to deg
# → 135 deg
```

See also: [atan](math.md#atan), [hypot](math.md#hypot)

### sinh

`sinh(x)`: hyperbolic sine

```zil
sinh(1)
# → 1.1752
```

See also: [cosh](math.md#cosh), [tanh](math.md#tanh)

### cosh

`cosh(x)`: hyperbolic cosine

```zil
cosh(1)
# → 1.54308
```

See also: [sinh](math.md#sinh), [tanh](math.md#tanh)

### tanh

`tanh(x)`: hyperbolic tangent

```zil
tanh(1)
# → 0.761594
```

See also: [sinh](math.md#sinh), [cosh](math.md#cosh)

### fixed

`fixed(x, digits)`: string with exactly that many decimals; keeps units

```zil
pi.fixed(2)
# → "3.14"
(5 km to mi).fixed(1)
# → "3.1 mi"
```

See also: [sci](math.md#sci), [commas](math.md#commas), [round](math.md#round)

### sci

`sci(x, digits?)`: string in scientific notation

```zil
123456.sci
# → "1.23456e5"
123456.sci(2)
# → "1.23e5"
```

See also: [fixed](math.md#fixed)

### percent

`percent(x, digits?)`: string as a percentage

```zil
0.256.percent
# → "25.6%"
(1/3).percent(1)
# → "33.3%"
```

See also: [fixed](math.md#fixed)

### commas

`commas(x, digits?)`: string with thousands separators

```zil
1234567.commas
# → "1,234,567"
1234.5.commas(2)
# → "1,234.50"
```

See also: [fixed](math.md#fixed)

### gcd

`gcd(a, b, ...) / gcd(list)`: greatest common divisor

```zil
gcd(12, 18)
# → 6
[12, 18, 27].gcd
# → 3
```

See also: [lcm](math.md#lcm)

### lcm

`lcm(a, b, ...) / lcm(list)`: least common multiple

```zil
lcm(4, 6)
# → 12
(1..=20).lcm
# → 232792560
```

See also: [gcd](math.md#gcd)

### is_prime

`is_prime(n)`: primality (Miller-Rabin; exact below 3e24)

```zil
is_prime(97)
# → true
is_prime(2 ** 61 - 1)
# → true
```

See also: [factors](math.md#factors)

### factors

`factors(n)`: prime factors, smallest first

```zil
360.factors
# → [2, 2, 2, 3, 3, 5]
factors(2 ** 32 + 1)
# → [641, 6700417]
```

See also: [is_prime](math.md#is_prime), [gcd](math.md#gcd)

### factorial

`factorial(n)`: n!, exact

```zil
factorial(5)
# → 120
factorial(30)
# → 265252859812191058636308480000000
```

See also: [choose](math.md#choose)

### choose

`choose(n, k)`: ways to pick k of n, exact

```zil
choose(5, 2)
# → 10
choose(52, 5)
# → 2598960
```

See also: [factorial](math.md#factorial)

### mod_pow

`mod_pow(b, e, m)`: b ** e % m without the huge power

```zil
mod_pow(2, 100, 7)
# → 2
mod_pow(3, 10 ** 18, 1000000007)
# → 246336683
```

See also: [gcd](math.md#gcd)

### popcount

`popcount(n)`: number of 1 bits (two's complement for negatives)

```zil
popcount(0b1011)
# → 3
popcount(-1)
# → 64
```

See also: [bit](math.md#bit)

### bit

`bit(n, i)`: bit i of n (0 is the lowest), as 0 or 1

```zil
bit(0b100, 2)
# → 1
```

See also: [set_bit](math.md#set_bit), [clear_bit](math.md#clear_bit), [popcount](math.md#popcount)

### set_bit

`set_bit(n, i)`: n with bit i set

```zil
set_bit(0b1, 4)
# → 0b10001
```

See also: [clear_bit](math.md#clear_bit), [bit](math.md#bit)

### clear_bit

`clear_bit(n, i)`: n with bit i cleared

```zil
clear_bit(0xff, 0)
# → 0xfe
```

See also: [set_bit](math.md#set_bit), [bit](math.md#bit)

### rotl

`rotl(n, k, width)`: rotate the low width bits left by k

```zil
rotl(0x81, 1, 8)
# → 0x3
rotl(0x80000000, 1, 32)
# → 0x1
```

See also: [rotr](math.md#rotr)

### rotr

`rotr(n, k, width)`: rotate the low width bits right by k

```zil
rotr(0x81, 1, 8)
# → 0xc0
```

See also: [rotl](math.md#rotl)

### byteswap

`byteswap(n, width)`: reverse the bytes of a width-bit number

```zil
byteswap(0x1234, 16)
# → 0x3412
byteswap(0x12345678, 32)
# → 0x78563412
```

See also: [rotl](math.md#rotl)

## More examples

### numbers

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
# prime factors
factors(2 ** 32 + 1)
# → [641, 6700417]
# is a Mersenne number prime
is_prime(2 ** 61 - 1)
# → true
# poker hands
choose(52, 5)
# → 2598960
# modular power
mod_pow(3, 1000, 7)
# → 4
# add a percent
80 + 15%
# → 92
# percent of
15% of 64.50
# → 9.675
# trig takes angle units
sin(30 deg)
# → 0.5
# radians back to degrees
atan2(1, 1) to deg
# → 45 deg
# digit sum
(2 ** 100).digits.sum
# → 115
# format specs
"{1234567.891:,.2f}"
# → "1,234,567.89"
```

### bits

```zil
# see the bits
0xf0 to bits
# → "0b1111_0000"
# results keep their base
0xff + 1
# → 0x100
# two's complement
-1 to hex(16)
# → 0xffff
# xor
0b1010 ^ 0b0110
# → 0b1100
# count set bits
popcount(0xff)
# → 8
# rotate within 8 bits
rotr(1, 1, 8)
# → 128
# swap byte order
byteswap(0x1234, 16)
# → 0x3412
# clear a bit
clear_bit(0xff, 0)
# → 0xfe
# hex with a prefix
"{255:#06x}"
# → "0x00ff"
# 32-bit binary
"{0xdeadbeef:032b}"
# → "11011110101011011011111011101111"
# base 36
255 to base(36)
# → 36#73
```
