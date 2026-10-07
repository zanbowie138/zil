# units.kitchen

cups to grams with ingredient densities, decibels, and CSS pixels (px)

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Units

| kind | names |
|---|---|
| length | px, pixel, pixels |

## Functions

| function | description |
|---|---|
| [`density(ingredient: str)`](#density) | a typical density in g/mL, so volume * density is mass; baking ingredients are spooned and leveled |
| [`db(ratio: num, kind?: str)`](#db) | a power ratio in decibels (10 log10); kind "amplitude" for voltage or pressure ratios (20 log10) |
| [`from_db(db: num, kind?: str)`](#from_db) | decibels back to a power ratio, or an amplitude ratio with "amplitude" |

### density

`density(ingredient: str)`: a typical density in g/mL, so volume * density is mass; baking ingredients are spooned and leveled

```zil
density("honey")
# → 1.42 g/mL
2 tbsp * density("butter") to g
# → 28.3906 g
```

See also: [db](../units/kitchen.md#db)

### db

`db(ratio: num, kind?: str)`: a power ratio in decibels (10 log10); kind "amplitude" for voltage or pressure ratios (20 log10)

```zil
db(2)
# → 3.0103
db(1000)
# → 30
db(2, "amplitude")
# → 6.0206
```

See also: [from_db](../units/kitchen.md#from_db), [log](../math.md#log)

### from_db

`from_db(db: num, kind?: str)`: decibels back to a power ratio, or an amplitude ratio with "amplitude"

```zil
from_db(3)
# → 1.99526
from_db(-6, "amplitude")
# → 0.501187
```

See also: [db](../units/kitchen.md#db)

## More examples

### kitchen

```zil
# a cup of flour
1 cup * density("flour") to g
# → 125.392 g
# 250 g of sugar in cups
250 g / density("sugar") to cup
# → 1.24316 cup
# a stick of butter
0.5 cup * density("butter") to g
# → 113.562 g
# twice the power
db(2)
# → 3.0103
# +3 dB louder means
from_db(3)
# → 1.99526
# a 1080p screen at 96 dpi
1080 px to in
# → 11.25 in
```
