# Run: zil examples/math.zil

# Exact arithmetic: no floating point surprises
print(0.1 + 0.2 == 0.3)
print(1/3 + 1/4 + 1/5 to frac)
print("2^200 = {2 ** 200}")
print("100! has {str(factorial(100)).len} digits")

# Number theory
print("primes under 50: {(2..50).filter(is_prime)}")
print("factors of 360: {factors(360)}")
print("gcd(1071, 462) = {gcd(1071, 462)}")

# Collatz: how long does 27 take to reach 1?
n = 27
steps = [n]
while n != 1 {
  n = if n % 2 == 0 { n // 2 } else { 3 * n + 1 }
  steps.push(n)
}
print("27 takes {steps.len - 1} steps and peaks at {steps.max}")
print(steps.sparkline)

# Calculus, numerically
f = |x| x ** 3 - 2 * x - 5
print("root of x^3 - 2x - 5: {root(f, 2, 3)}")
print("area under sin from 0 to pi: {integrate(sin, 0, pi)}")

# Complex numbers and linear systems
print("(2 + 3i) * (1 - 1i) = {(2 + 3i) * (1 - 1i)}")
print("x + y = 10, x - y = 2 gives {solve([[1, 1], [1, -1]], [10, 2])}")

print(plot(|x| sin(x) * x, -10, 10))
