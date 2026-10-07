# math.numtheory

gcd, primes, factoring, factorials, modular powers; exact at any size

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`gcd(a, b, ...) / gcd(list)`](#gcd) | greatest common divisor |
| [`lcm(a, b, ...) / lcm(list)`](#lcm) | least common multiple |
| [`is_prime(n)`](#is_prime) | primality (Miller-Rabin; exact below 3e24) |
| [`factors(n)`](#factors) | prime factors, smallest first |
| [`factorial(n)`](#factorial) | n!, exact |
| [`choose(n, k)`](#choose) | ways to pick k of n, exact |
| [`mod_pow(b, e, m)`](#mod_pow) | b ** e % m without the huge power |

### gcd

`gcd(a, b, ...) / gcd(list)`: greatest common divisor

```zil
gcd(12, 18)
# → 6
[12, 18, 27].gcd
# → 3
```

See also: [lcm](../math/numtheory.md#lcm)

### lcm

`lcm(a, b, ...) / lcm(list)`: least common multiple

```zil
lcm(4, 6)
# → 12
(1..=20).lcm
# → 232792560
```

See also: [gcd](../math/numtheory.md#gcd)

### is_prime

`is_prime(n)`: primality (Miller-Rabin; exact below 3e24)

```zil
is_prime(97)
# → true
is_prime(2 ** 61 - 1)
# → true
```

See also: [factors](../math/numtheory.md#factors)

### factors

`factors(n)`: prime factors, smallest first

```zil
360.factors
# → [2, 2, 2, 3, 3, 5]
factors(2 ** 32 + 1)
# → [641, 6700417]
```

See also: [is_prime](../math/numtheory.md#is_prime), [gcd](../math/numtheory.md#gcd)

### factorial

`factorial(n)`: n!, exact

```zil
factorial(5)
# → 120
factorial(30)
# → 265252859812191058636308480000000
```

See also: [choose](../math/numtheory.md#choose)

### choose

`choose(n, k)`: ways to pick k of n, exact

```zil
choose(5, 2)
# → 10
choose(52, 5)
# → 2598960
```

See also: [factorial](../math/numtheory.md#factorial)

### mod_pow

`mod_pow(b, e, m)`: b ** e % m without the huge power

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
