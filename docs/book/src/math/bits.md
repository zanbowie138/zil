# math.bits

count, test, set, rotate and swap bits

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

### conversions

```zil
# to bits
0xf0 to bits
# → "0b1111_0000"
```

## Functions

| function | description |
|---|---|
| [`popcount(n)`](#popcount) | number of 1 bits (two's complement for negatives) |
| [`bit(n, i)`](#bit) | bit i of n (0 is the lowest), as 0 or 1 |
| [`set_bit(n, i)`](#set_bit) | n with bit i set |
| [`clear_bit(n, i)`](#clear_bit) | n with bit i cleared |
| [`rotl(n, k, width)`](#rotl) | rotate the low width bits left by k |
| [`rotr(n, k, width)`](#rotr) | rotate the low width bits right by k |
| [`byteswap(n, width)`](#byteswap) | reverse the bytes of a width-bit number |

### popcount

`popcount(n)`: number of 1 bits (two's complement for negatives)

```zil
popcount(0b1011)
# → 3
popcount(-1)
# → 64
```

See also: [bit](../math/bits.md#bit)

### bit

`bit(n, i)`: bit i of n (0 is the lowest), as 0 or 1

```zil
bit(0b100, 2)
# → 1
```

See also: [set_bit](../math/bits.md#set_bit), [clear_bit](../math/bits.md#clear_bit), [popcount](../math/bits.md#popcount)

### set_bit

`set_bit(n, i)`: n with bit i set

```zil
set_bit(0b1, 4)
# → 0b10001
```

See also: [clear_bit](../math/bits.md#clear_bit), [bit](../math/bits.md#bit)

### clear_bit

`clear_bit(n, i)`: n with bit i cleared

```zil
clear_bit(0xff, 0)
# → 0xfe
```

See also: [set_bit](../math/bits.md#set_bit), [bit](../math/bits.md#bit)

### rotl

`rotl(n, k, width)`: rotate the low width bits left by k

```zil
rotl(0x81, 1, 8)
# → 0x3
rotl(0x80000000, 1, 32)
# → 0x1
```

See also: [rotr](../math/bits.md#rotr)

### rotr

`rotr(n, k, width)`: rotate the low width bits right by k

```zil
rotr(0x81, 1, 8)
# → 0xc0
```

See also: [rotl](../math/bits.md#rotl)

### byteswap

`byteswap(n, width)`: reverse the bytes of a width-bit number

```zil
byteswap(0x1234, 16)
# → 0x3412
byteswap(0x12345678, 32)
# → 0x78563412
```

See also: [rotl](../math/bits.md#rotl)

## More examples

### bits

```zil
# see the bits
0xf0 to bits
# → "0b1111_0000"
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
```
