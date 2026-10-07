# math.bases

hex, binary, octal and any base 2-36; digits of a number

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

### conversions

```zil
# to hex bin oct dec
255 to bin
# → 0b11111111
# to hex(bits), to base(b)
-1 to hex(16)
# → 0xffff
```

## Functions

| function | description |
|---|---|
| [`hex(v, bits?)`](#hex) | same as `v to hex` / `v to hex(bits)`; strings become hex bytes |
| [`bin(v, bits?)`](#bin) | same as `v to bin` / `v to bin(bits)` |
| [`oct(v, bits?)`](#oct) | same as `v to oct` |
| [`dec(v, bits?)`](#dec) | same as `v to dec`; with bits, reads two's complement as unsigned |
| [`base(v, b)`](#base) | same as `v to base(b)`, any base 2-36 |
| [`digits(n)`](#digits) | list of digits in the number's own base |
| [`from_digits(list, base?)`](#from_digits) | build a number from digits, kept in that base |

### hex

`hex(v, bits?)`: same as `v to hex` / `v to hex(bits)`; strings become hex bytes

```zil
hex(255)
# → 0xff
hex(-1, 16)
# → 0xffff
hex("hi")
# → "6869"
```

See also: [bin](../math/bases.md#bin), [base](../math/bases.md#base), [int](../core.md#int)

### bin

`bin(v, bits?)`: same as `v to bin` / `v to bin(bits)`

```zil
bin(10)
# → 0b1010
bin(5, 8)
# → 0b00000101
```

See also: [hex](../math/bases.md#hex), [oct](../math/bases.md#oct)

### oct

`oct(v, bits?)`: same as `v to oct`

```zil
oct(8)
# → 0o10
```

See also: [hex](../math/bases.md#hex), [bin](../math/bases.md#bin)

### dec

`dec(v, bits?)`: same as `v to dec`; with bits, reads two's complement as unsigned

```zil
dec(0xff)
# → 255
dec(-1, 8)
# → 255
```

See also: [hex](../math/bases.md#hex), [int](../core.md#int)

### base

`base(v, b)`: same as `v to base(b)`, any base 2-36

```zil
base(35, 36)
# → 36#z
base(10, 3)
# → 3#101
```

See also: [hex](../math/bases.md#hex), [digits](../math/bases.md#digits)

### digits

`digits(n)`: list of digits in the number's own base

```zil
1234.digits
# → [1, 2, 3, 4]
0b1011.digits
# → [1, 0, 1, 1]
```

See also: [from_digits](../math/bases.md#from_digits)

### from_digits

`from_digits(list, base?)`: build a number from digits, kept in that base

```zil
[1, 2, 3].from_digits
# → 123
[1, 0, 1, 1].from_digits(2)
# → 0b1011
```

See also: [digits](../math/bases.md#digits)

## More examples

### bases

```zil
# number bases
255 to bin
# → 0b11111111
# any base back to decimal
36#zz to dec
# → 1295
# hex of a big int
factorial(20) to hex
# → 0x21c3677c82b40000
# results keep their base
0xff + 1
# → 0x100
# two's complement
-1 to hex(16)
# → 0xffff
# base 36
255 to base(36)
# → 36#73
# digit sum
(2 ** 100).digits.sum
# → 115
```
