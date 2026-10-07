# math.numtheory

gcd, primes, factoring, factorials, modular powers; exact at any size

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`gcd(a: int, b: int, ...) / gcd(xs: list)`](#gcd) | greatest common divisor |
| [`lcm(a: int, b: int, ...) / lcm(xs: list)`](#lcm) | least common multiple |
| [`is_prime(n: int)`](#is_prime) | primality (Miller-Rabin; exact below 3e24) |
| [`factors(n: int)`](#factors) | prime factors, smallest first |
| [`factorial(n: int)`](#factorial) | n!, exact |
| [`choose(n: int, k: int)`](#choose) | ways to pick k of n, exact |
| [`mod_pow(b: int, e: int, m: int)`](#mod_pow) | b ** e % m without the huge power |

### gcd

`gcd(a: int, b: int, ...) / gcd(xs: list)`: greatest common divisor

```zil
gcd(12, 18)
# → 6
[12, 18, 27].gcd
# → 3
```

See also: [lcm](../math/numtheory.md#lcm)

### lcm

`lcm(a: int, b: int, ...) / lcm(xs: list)`: least common multiple

```zil
lcm(4, 6)
# → 12
(1..=20).lcm
# → 232792560
```

See also: [gcd](../math/numtheory.md#gcd)

### is_prime

`is_prime(n: int)`: primality (Miller-Rabin; exact below 3e24)

```zil
is_prime(97)
# → true
is_prime(2 ** 61 - 1)
# → true
```

See also: [factors](../math/numtheory.md#factors)

### factors

`factors(n: int)`: prime factors, smallest first

```zil
360.factors
# → [2, 2, 2, 3, 3, 5]
factors(2 ** 32 + 1)
# → [641, 6700417]
```

See also: [is_prime](../math/numtheory.md#is_prime), [gcd](../math/numtheory.md#gcd)

### factorial

`factorial(n: int)`: n!, exact

```zil
factorial(5)
# → 120
factorial(30)
# → 265252859812191058636308480000000
```

See also: [choose](../math/numtheory.md#choose)

### choose

`choose(n: int, k: int)`: ways to pick k of n, exact

```zil
choose(5, 2)
# → 10
choose(52, 5)
# → 2598960
```

See also: [factorial](../math/numtheory.md#factorial)

### mod_pow

`mod_pow(b: int, e: int, m: int)`: b ** e % m without the huge power

```zil
mod_pow(2, 100, 7)
# → 2
mod_pow(3, 10 ** 18, 1000000007)
# → 246336683
```

See also: [gcd](../math/numtheory.md#gcd)

## More examples

### numtheory

```zil
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
```
