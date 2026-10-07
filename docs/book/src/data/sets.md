# data.sets

unique values in insertion order: union, intersection, difference

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### types

```zil
# set
set(1, 2, 3)
```

### operators

```zil
# a | b  union
set(1, 2) | set(2, 3)
# → set(1, 2, 3)
# a & b  intersection
set(1, 2) & set(2, 3)
# → set(2)
# a - b  difference
set(1, 2) - set(2, 3)
# → set(1)
# a ^ b  symmetric difference
set(1, 2) ^ set(2, 3)
# → set(1, 3)
# x in s
2 in set(1, 2)
# → true
```

### holds

```zil
# int, str, bool and nil only
```

### pretty

```zil
# long: one item per line, indented, items pretty
# short: one line, items short
pretty(set(1500, "a"), "short")
# → "set(1.5K, \"a\")"
```

## Functions

| function | description |
|---|---|
| [`set(xs: list\|str\|set) / set(a?: any, ...)`](#set) | a set of a list's items (or a string's characters), or of the arguments |
| [`is_subset(a: set, b: set)`](#is_subset) | true if every item of a is in b |
| [`is_superset(a: set, b: set)`](#is_superset) | true if a has every item of b |
| [`is_disjoint(a: set, b: set)`](#is_disjoint) | true if a and b share nothing |
| [`add(s: set, v: any)`](#add) | add v in place and return the set |
| [`remove(s: set, v: any)`](#remove) | remove v in place (if there) and return the set |

### set

`set(xs: list|str|set) / set(a?: any, ...)`: a set of a list's items (or a string's characters), or of the arguments

```zil
[3, 1, 3, 2].set
# → set(3, 1, 2)
set(1, 2)
"hello" to set
# → set("h", "e", "l", "o")
```

See also: [unique](../data.md#unique), [list](../core.md#list)

### is_subset

`is_subset(a: set, b: set)`: true if every item of a is in b

```zil
set(1, 2).is_subset(set(1, 2, 3))
# → true
```

See also: [is_superset](../data/sets.md#is_superset)

### is_superset

`is_superset(a: set, b: set)`: true if a has every item of b

```zil
set(1, 2, 3).is_superset(set(1, 5))
# → false
```

See also: [is_subset](../data/sets.md#is_subset)

### is_disjoint

`is_disjoint(a: set, b: set)`: true if a and b share nothing

```zil
set(1, 2).is_disjoint(set(3))
# → true
```

See also: [is_subset](../data/sets.md#is_subset)

### add

`add(s: set, v: any)`: add v in place and return the set

```zil
set(1).add(2)
# → set(1, 2)
```

See also: [remove](../data/sets.md#remove), [push](../data/lists.md#push)

### remove

`remove(s: set, v: any)`: remove v in place (if there) and return the set

```zil
set(1, 2).remove(1)
# → set(2)
```

See also: [add](../data/sets.md#add), [del](../data/maps.md#del)

## More examples

### sets

```zil
# distinct words
"the cat and the hat".split.set
# → set("the", "cat", "and", "hat")
# in both
set(1, 2, 3) & set(2, 3, 4)
# → set(2, 3)
# in either, not both
set(1, 2, 3) ^ set(2, 3, 4)
# → set(1, 4)
# order doesn't matter
set(1, 2) == set(2, 1)
# → true
```
