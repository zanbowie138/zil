# units

numbers with units, combined and converted; currencies use live rates

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### types

```zil
# quantity
5 km
```

### operators

```zil
# qty ± qty
1 km + 300 m
# → 1.3 km
# qty * / qty
100 km / 2 h
# → 50 km/h
# qty ** int
(3 m) ** 2
# → 9 m^2
# qty < qty
1 mi > 1 km
# → true
```

### pretty

```zil
# long: best prefix, thousands separators
pretty(12345678 m)
# → "12,345.7 km"
# short: best prefix, K M B T
pretty(12345678 m, "short")
# → "12.3K km"
# durations split into d h min s, money keeps its own display: see time, money
```

### conversions

```zil
# to unit
5 km to mi
# → 3.10686 mi
# in unit
5 km in mi
# → 3.10686 mi
# to unit unit
1.8 m to ft in
# → "5 ft 10.8661 in"
# to compound
100 km / 2 h to mph
# → 31.0686 mph
```

### your own units

```zil
# unit sprint = 2 wk        then  3 sprint to d → 42 d
# unit pizza                a new base unit, for counting
# unit slice = pizza / 8    then  3 slice to pizza → 0.375 pizza
```

## Units

| kind | names |
|---|---|
| length | m, meter, meters, metre, metres; km, kilometer, kilometers; cm, centimeter, centimeters; mm, millimeter, millimeters; µm, um, μm, micrometer, micrometers, micron; nm, nanometer, nanometers; mi, mile, miles; yd, yard, yards; ft, foot, feet; in, inch, inches; nmi; au; ly, lightyear, lightyears |
| mass | kg, kilogram, kilograms; g, gram, grams; mg, milligram, milligrams; µg, ug, μg, microgram, micrograms; t, tonne, tonnes; lb, lbs, pound, pounds; oz, ounce, ounces; st, stone |
| time | s, sec, secs, second, seconds; ms, millisecond, milliseconds; µs, us, μs, microsecond, microseconds; ns, nanosecond, nanoseconds; min, mins, minute, minutes; h, hr, hrs, hour, hours; d, day, days; wk, week, weeks; mo, month, months; yr, year, years; workday, workdays; workwk, workweek, workweeks; workmo, workmonth, workmonths; workyr, workyear, workyears |
| temperature | K, kelvin; C, celsius, degC; F, fahrenheit, degF |
| volume | L, l, liter, liters, litre, litres; mL, ml, milliliter, milliliters; gal, gallon, gallons; qt, quart, quarts; pt, pint, pints; cup, cups; floz; tbsp, tablespoon, tablespoons; tsp, teaspoon, teaspoons |
| area | ha, hectare, hectares; acre, acres |
| speed | kph, kmh; mph; kn, knot, knots |
| data | bit, bits; B, byte, bytes; KB; MB; GB; TB; PB; KiB; MiB; GiB; TiB; PiB; kbit, Kb; Mbit, Mb; Gbit, Gb |
| rate | bps; kbps; Mbps; Gbps |
| energy | J, joule, joules; kJ; MJ; cal, calorie, calories; kcal, Cal; Wh; kWh; eV; BTU, btu |
| power | W, watt, watts; kW; MW; hp, horsepower |
| pressure | Pa, pascal; kPa; MPa; bar; atm; psi; mmHg |
| force | N, newton, newtons; kN; lbf |
| angle | rad, radian, radians; deg, degree, degrees; turn, turns |
| current | A, amp, amps, ampere, amperes; mA, milliamp, milliamps; µA, uA, μA; kA |
| voltage | V, volt, volts; mV; µV, uV, μV; kV |
| resistance | Ω, ohm, ohms; mΩ, mohm; kΩ, kohm; MΩ, Mohm |
| charge | coulomb, coulombs; mAh; Ah |
| capacitance | farad, farads; mF; µF, uF, μF; nF; pF |
| conductance | S, siemens; mS |
| magnetic_flux | Wb, weber, webers; mWb |
| magnetic_field | T, tesla, teslas; mT; µT, uT, μT |
| inductance | H, henry, henries; mH; µH, uH, μH |
| frequency | Hz, hertz; kHz; MHz; GHz; rpm |
| amount | mol, mole, moles; mmol; µmol, umol, μmol; kmol |
| concentration | M, molar; mM, millimolar; µM, uM, μM, micromolar |
| light | lm, lumen, lumens; cd, candela, candelas |
| illuminance | lx, lux; fc, footcandle, footcandles |
| dose | Sv, sievert, sieverts; mSv, millisievert, millisieverts; µSv, uSv, μSv, microsievert, microsieverts |

## Submodules

| module | about |
|---|---|
| [constants](units/constants.md) | physical constants (CODATA 2018) as quantities; `h` is still hours, so Planck's constant is `h_planck` |
| [money](units/money.md) | currencies shown as money, pay, interest, loans, splitting bills |
| [goofy](units/goofy.md) | bananas for scale, smoots, fortnights; every item also works as item_for_scale |
| [kitchen](units/kitchen.md) | cups to grams with ingredient densities, decibels, and CSS pixels (px) |

## Functions

| function | description |
|---|---|
| [`simplify(q: quantity)`](#simplify) | the quantity in the prefixed unit of the same family that reads best, like `1.2 m` or `3.6 kW`; also `to best` |
| [`all_units()`](#all_units) | every unit as a table: {name, full, desc, aliases, kind, si, source}; source is units, goofy, kitchen, currency or user |

### simplify

`simplify(q: quantity)`: the quantity in the prefixed unit of the same family that reads best, like `1.2 m` or `3.6 kW`; also `to best`

```zil
0.0012 km to best
# → 1.2 m
3600 J/s to best
# → 3.6 kW
1 kg*m^2/s^2 to best
# → 1 J
1.5e9 B.simplify
# → 1.5 GB
```

### all_units

`all_units()`: every unit as a table: {name, full, desc, aliases, kind, si, source}; source is units, goofy, kitchen, currency or user

```zil
all_units().len
# → 266
all_units().filter(|u| u.kind == "length").map(|u| u.name).take(5)
# → ["m", "km", "cm", "mm", "µm"]
all_units().filter(|u| u.source == "goofy")[0]
# → {name: "banana", full: "banana", desc: "a typical banana, 17.8 cm; the internet's favorite scale", aliases: ["bananas"], kind: "length", si: 0.178 m, source: "goofy"}
```

See also: [simplify](units.md#simplify)

## More examples

### units

```zil
# speed from distance and time
100 km / 2 h to mph
# → 31.0686 mph
# feet and inches
1.8 m to ft in
# → "5 ft 10.8661 in"
# mixed units add
5 ft 11 in to cm
# → 180.34 cm
# drive time
300 mi / 65 mph to h min
# → "4 h 36.9231 min"
# download time
4 GB / 100 Mbps to min
# → 5.33333 min
# why a 1 TB disk shows less
1 TB to GiB
# → 931.323 GiB
# area
3 m * 4 m to ft^2
# → 129.167 ft^2
# running pace
1 h / 10 km to min/mi
# → 9.65606 min/mi
# force
9.81 m/s^2 * 70 kg to N
# → 686.7 N
# a month of a 60 W bulb
60 W * 8 h * 30 to kWh
# → 14.4 kWh
# temperature
101 F to C
# → 38.3333 C
# 20 USD to EUR   (currencies use live rates)
```
