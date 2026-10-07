# math.linalg

dot and cross products, norms, matrix multiply, transpose, determinant, inverse, linear systems; exact with fractions

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`dot(a: list, b: list)`](#dot) | dot product of two vectors of the same length |
| [`cross(a: list, b: list)`](#cross) | cross product of two 3-vectors |
| [`norm(v: list)`](#norm) | length of a vector |
| [`transpose(m: list)`](#transpose) | swap rows and columns |
| [`matmul(a: list, b: list)`](#matmul) | matrix product; a's column count must match b's row count |
| [`det(m: list)`](#det) | determinant of a square matrix |
| [`inv(m: list)`](#inv) | inverse of a square matrix; exact for ints and fractions |
| [`solve(a: list, b: list)`](#solve) | x such that a × x = b, for square a |

### dot

`dot(a: list, b: list)`: dot product of two vectors of the same length

```zil
dot([1, 2, 3], [4, 5, 6])
# → 32
```

See also: [cross](../math/linalg.md#cross), [norm](../math/linalg.md#norm)

### cross

`cross(a: list, b: list)`: cross product of two 3-vectors

```zil
cross([1, 0, 0], [0, 1, 0])
# → [0, 0, 1]
```

See also: [dot](../math/linalg.md#dot)

### norm

`norm(v: list)`: length of a vector

```zil
norm([3, 4])
# → 5
norm([1 m, 1 m])
# → 1.41421 m
```

See also: [dot](../math/linalg.md#dot), [hypot](../math.md#hypot)

### transpose

`transpose(m: list)`: swap rows and columns

```zil
[[1, 2, 3], [4, 5, 6]].transpose
# → [[1, 4], [2, 5], [3, 6]]
```

See also: [matmul](../math/linalg.md#matmul)

### matmul

`matmul(a: list, b: list)`: matrix product; a's column count must match b's row count

```zil
matmul([[1, 2], [3, 4]], [[5], [6]])
# → [[17], [39]]
```

See also: [transpose](../math/linalg.md#transpose), [inv](../math/linalg.md#inv)

### det

`det(m: list)`: determinant of a square matrix

```zil
det([[1, 2], [3, 4]])
# → -2
```

See also: [inv](../math/linalg.md#inv), [solve](../math/linalg.md#solve)

### inv

`inv(m: list)`: inverse of a square matrix; exact for ints and fractions

```zil
inv([[4, 7], [2, 6]])
# → [[0.6, -0.7], [-0.2, 0.4]]
```

See also: [det](../math/linalg.md#det), [solve](../math/linalg.md#solve)

### solve

`solve(a: list, b: list)`: x such that a × x = b, for square a

```zil
solve([[2, 1], [1, 3]], [3, 5])
# → [0.8, 1.4]
solve([[1, 1, 1], [0, 2, 5], [2, 5, -1]], [6, -4, 27])
# → [5, 3, -2]
```

See also: [inv](../math/linalg.md#inv), [det](../math/linalg.md#det)

## More examples

### linalg

```zil
# solve 2x + y = 3, x + 3y = 5
solve([[2, 1], [1, 3]], [3, 5]).map(frac)
# → [4/5, 7/5]
# exact inverse
inv([[2, 1], [1, 1]])
# → [[1, -1], [-1, 2]]
# rotate (1, 0) by 90°
matmul([[cos(90 deg), -sin(90 deg)], [sin(90 deg), cos(90 deg)]], [[1], [0]])
# → [[6.12323e-17], [1]]
# distance between points
norm([3 m, 4 m])
# → 5 m
```
