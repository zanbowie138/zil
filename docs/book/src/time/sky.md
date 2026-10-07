# time.sky

sunrise, sunset, day length and moon phase for a place and date; good to about a minute

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### coordinates

```zil
# latitude north and longitude east are positive, in degrees
# or name a place: a city or country, or a map with lat and lon
sunset("Oslo", date(2026, 6, 21))
# → 2026-06-21 15:43:43 -05:00
# times come back in the date's time zone
```

## Functions

| function | description |
|---|---|
| [`sunrise(lat: num, lon: num, day?: date) / sunrise(place: str\|map, day?: date)`](#sunrise) | when the sun rises there on that day (default today), in the day's time zone; nil during polar night or midnight sun |
| [`sunset(lat: num, lon: num, day?: date) / sunset(place: str\|map, day?: date)`](#sunset) | when the sun sets there on that day (default today); nil if it doesn't |
| [`day_length(lat: num, lon: num, day?: date) / day_length(place: str\|map, day?: date)`](#day_length) | time from sunrise to sunset: 0 h in polar night, 24 h under the midnight sun |
| [`moon_phase(day?: date)`](#moon_phase) | the moon's phase and how much of it is lit |

### sunrise

`sunrise(lat: num, lon: num, day?: date) / sunrise(place: str|map, day?: date)`: when the sun rises there on that day (default today), in the day's time zone; nil during polar night or midnight sun

```zil
sunrise(40.71, -74.01, date(2026, 3, 20))
# → 2026-03-20 05:59:37 -05:00
sunrise("Reykjavik").hour
# → 7
```

See also: [sunset](../time/sky.md#sunset), [day_length](../time/sky.md#day_length)

### sunset

`sunset(lat: num, lon: num, day?: date) / sunset(place: str|map, day?: date)`: when the sun sets there on that day (default today); nil if it doesn't

```zil
sunset(51.5, -0.13, date(2026, 12, 21))
# → 2026-12-21 09:53:16 -06:00
sunset("Oslo", date(2026, 6, 21))
# → 2026-06-21 15:43:43 -05:00
```

See also: [sunrise](../time/sky.md#sunrise), [day_length](../time/sky.md#day_length)

### day_length

`day_length(lat: num, lon: num, day?: date) / day_length(place: str|map, day?: date)`: time from sunrise to sunset: 0 h in polar night, 24 h under the midnight sun

```zil
day_length(0, 0, date(2026, 3, 20)) to h min
# → "12 h 6.66405 min"
day_length(70, 25, date(2026, 6, 21))
# → 24 h
```

See also: [sunrise](../time/sky.md#sunrise), [sunset](../time/sky.md#sunset)

### moon_phase

`moon_phase(day?: date)`: the moon's phase and how much of it is lit

```zil
moon_phase()
# → "🌘 waning crescent, 13% lit"
moon_phase(date(2026, 1, 3))
# → "🌕 full moon, 100% lit"
```

See also: [sunrise](../time/sky.md#sunrise)

## More examples

### sky

```zil
# sunset in Paris on the solstice
sunset(48.86, 2.35, date("2026-06-21T12:00+02:00") to "Europe/Paris")
# → 2026-06-21 21:57:44 +02:00
# the longest day in Oslo
day_length(59.91, 10.75, date(2026, 6, 21)) to h min
# → "18 h 49.9557 min"
# no sunrise at the pole in winter
sunrise(89, 0, date(2026, 12, 21))
# → nil
# moon on the 2026 Christmas
moon_phase(date(2026, 12, 25))
# → "🌕 full moon, 99% lit"
```
