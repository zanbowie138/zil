# time.zones

the same moment in UTC, local time, any IANA zone or a city; world clocks

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### conversions

```zil
# to UTC, to local
date("2026-12-25 18:30") to UTC
# → 2026-12-26 00:30:00 +00:00
# to "Zone/Name"
date("2026-12-25 18:30") to "Asia/Tokyo"
# → 2026-12-26 09:30:00 +09:00
# to "City" or "Country"
date("2026-12-25 18:30") to "Sao Paulo"
# → 2026-12-25 21:30:00 -03:00
```

## Functions

| function | description |
|---|---|
| [`utc(d: date)`](#utc) | the same moment in UTC; same as `d to UTC` (`d to "Asia/Tokyo"` for any zone) |
| [`local(d: date)`](#local) | the same moment in the system time zone; same as `d to local` |
| [`clock(places: list) / clock(place: str)`](#clock) | the time now in each zone or city |

### utc

`utc(d: date)`: the same moment in UTC; same as `d to UTC` (`d to "Asia/Tokyo"` for any zone)

```zil
date(0) to UTC
# → 1970-01-01 00:00:00 +00:00
date(0).utc.year
# → 1970
```

See also: [local](../time/zones.md#local), [date](../time.md#date)

### local

`local(d: date)`: the same moment in the system time zone; same as `d to local`

```zil
(date(0) to UTC).local.year
# → 1969
```

See also: [utc](../time/zones.md#utc)

### clock

`clock(places: list) / clock(place: str)`: the time now in each zone or city

```zil
clock(["Tokyo", "London"])
# → {Tokyo: 2026-10-08 05:45:19 +09:00, London: 2026-10-07 21:45:19 +01:00}
clock("Asia/Kolkata").hour >= 0
# → true
```

See also: [utc](../time/zones.md#utc), [local](../time/zones.md#local)

## More examples

### zones

```zil
# time zones
date("2026-12-25 18:30") to "Asia/Tokyo"
# → 2026-12-26 09:30:00 +09:00
# meeting time abroad
5pm to "London"
# → 2026-10-07 23:00:00 +01:00
# world clock
clock(["Tokyo", "London", "New York"])
# → {Tokyo: 2026-10-08 05:45:19 +09:00, London: 2026-10-07 21:45:19 +01:00, New York: 2026-10-07 16:45:19 -04:00}
```
