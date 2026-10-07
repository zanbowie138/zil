# time

dates: parsing, fields, date math, durations, relative text

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

### types

```zil
# date
date("2026-12-25 18:30")
# → 2026-12-25 18:30:00 -06:00
```

### names

```zil
# now today tomorrow yesterday
```

### operators

```zil
# date ± time
date("2026-01-31") + 1 mo
# → 2026-02-28
# date - date
date("2027-01-01") - date("2026-12-25")
# → 7 d
# date < date
yesterday < now
# → true
```

### conversions

```zil
# to unix
date("2026-12-25") to unix
# → 1798178400
```

## Submodules

| module | about |
|---|---|
| [calendar_math](time/calendar_math.md) | periods, weekdays, business days, week numbers, month grids; weeks start Monday |
| [zones](time/zones.md) | the same moment in UTC, local time or any IANA zone |

## Functions

| function | description |
|---|---|
| [`date(s) / date(s, fmt) / date(y, m, d, h?, min?, s?) / date(unix)`](#date) | parse ISO, US (12/25/2026), written (Dec 25 2026) or natural language dates |
| [`year(d)`](#year) | year of a date |
| [`month(d)`](#month) | month of a date (1-12) |
| [`day(d)`](#day) | day of the month |
| [`hour(d)`](#hour) | hour of a date |
| [`minute(d)`](#minute) | minute of a date |
| [`second(d)`](#second) | second of a date |
| [`weekday(d)`](#weekday) | day name |
| [`format(d, fmt) / format(fmt, args...)`](#format) | format a date with strftime codes, or numbers printf-style |
| [`with(d, fields)`](#with) | same date with fields replaced: year month day hour minute second |
| [`is_weekend(d)`](#is_weekend) | Saturday or Sunday |
| [`is_weekday(d)`](#is_weekday) | Monday through Friday |
| [`is_today(d)`](#is_today) | same calendar day as now |
| [`is_past(d)`](#is_past) | before now |
| [`is_future(d)`](#is_future) | after now |
| [`age(d)`](#age) | whole years since d |
| [`diff(a, b)`](#diff) | calendar difference from a to b |
| [`relative(d)`](#relative) | "in 3 days", "2 hours ago" |
| [`parts(duration)`](#parts) | duration in up to three of d, h, min, s; or `to d h min` for chosen units |
| [`unix(d)`](#unix) | seconds since 1970-01-01 UTC; same as `d to unix` |

### date

`date(s) / date(s, fmt) / date(y, m, d, h?, min?, s?) / date(unix)`: parse ISO, US (12/25/2026), written (Dec 25 2026) or natural language dates

```zil
date("2026-12-25")
# → 2026-12-25
date("next friday at 5pm")
# → 2026-10-09 17:00:00 -05:00
date("3 days ago")
# → 2026-10-03 23:30:59 -05:00
date("25.12.2026", "%d.%m.%Y")
# → 2026-12-25
date(2026, 12, 25, 18, 30)
# → 2026-12-25 18:30:00 -06:00
date(0)
# → 1969-12-31 18:00:00 -06:00
```

See also: [format](time.md#format), [with](time.md#with), [start_of](time/calendar_math.md#start_of)

### year

`year(d)`: year of a date

```zil
now.year
# → 2026
```

See also: [month](time.md#month), [day](time.md#day)

### month

`month(d)`: month of a date (1-12)

```zil
now.month
# → 10
```

See also: [year](time.md#year), [day](time.md#day)

### day

`day(d)`: day of the month

```zil
now.day
# → 6
```

See also: [month](time.md#month), [weekday](time.md#weekday)

### hour

`hour(d)`: hour of a date

```zil
now.hour
# → 23
```

See also: [minute](time.md#minute), [second](time.md#second)

### minute

`minute(d)`: minute of a date

```zil
now.minute
# → 30
```

See also: [hour](time.md#hour), [second](time.md#second)

### second

`second(d)`: second of a date

```zil
now.second
# → 59
```

See also: [hour](time.md#hour), [minute](time.md#minute)

### weekday

`weekday(d)`: day name

```zil
date("2026-12-25").weekday
# → "Friday"
```

See also: [day](time.md#day), [format](time.md#format)

### format

`format(d, fmt) / format(fmt, args...)`: format a date with strftime codes, or numbers printf-style

```zil
now.format("%B %d, %Y")
# → "October 06, 2026"
now.format("%H:%M")
# → "23:30"
format("%5.2f%%", 12.345)
# → "12.35%"
```

See also: [date](time.md#date), [fixed](math/formatting.md#fixed)

### with

`with(d, fields)`: same date with fields replaced: year month day hour minute second

```zil
today.with({day: 1})
# → 2026-10-01
now.with({hour: 9, minute: 0})
# → 2026-10-06 09:00:59 -05:00
```

See also: [start_of](time/calendar_math.md#start_of), [date](time.md#date)

### is_weekend

`is_weekend(d)`: Saturday or Sunday

```zil
date("2026-10-10").is_weekend
# → true
```

See also: [is_weekday](time.md#is_weekday), [add_workdays](time/calendar_math.md#add_workdays)

### is_weekday

`is_weekday(d)`: Monday through Friday

```zil
today.is_weekday
# → true
```

See also: [is_weekend](time.md#is_weekend)

### is_today

`is_today(d)`: same calendar day as now

```zil
now.is_today
# → true
tomorrow.is_today
# → false
```

See also: [is_past](time.md#is_past)

### is_past

`is_past(d)`: before now

```zil
yesterday.is_past
# → true
```

See also: [is_future](time.md#is_future), [is_today](time.md#is_today)

### is_future

`is_future(d)`: after now

```zil
tomorrow.is_future
# → true
```

See also: [is_past](time.md#is_past)

### age

`age(d)`: whole years since d

```zil
date("1990-06-15").age
# → 36
```

See also: [diff](time.md#diff)

### diff

`diff(a, b)`: calendar difference from a to b

```zil
diff(date("2025-08-03"), date("2026-10-06 04:00"))
# → "1 yr 2 mo 3 d 4 h"
```

See also: [age](time.md#age), [relative](time.md#relative), [parts](time.md#parts)

### relative

`relative(d)`: "in 3 days", "2 hours ago"

```zil
date("2026-12-25").relative
# → "in 3 months"
(now - 3 h).relative
# → "3 hours ago"
```

See also: [diff](time.md#diff), [parts](time.md#parts)

### parts

`parts(duration)`: duration in up to three of d, h, min, s; or `to d h min` for chosen units

```zil
(date("2026-12-25") - date("2026-10-06 14:24")).parts
# → "79 d 10 h 36 min"
5000 s.parts
# → "1 h 23 min 20 s"
```

See also: [relative](time.md#relative), [diff](time.md#diff)

### unix

`unix(d)`: seconds since 1970-01-01 UTC; same as `d to unix`

```zil
date(0).unix
# → 0
date(86400) to unix
# → 86400
```

See also: [date](time.md#date)

## More examples

### time

```zil
# day of the week
date("2026-12-25").weekday
# → "Friday"
# month math clamps to the end
date("2026-01-31") + 1 mo
# → 2026-02-28
# plain-English dates
date("next friday")
# → 2026-10-09
# countdown
(date("2027-01-01") - now).parts
# → "86 d 1 h 29 min"
# relative time
(now - 3 h).relative
# → "3 hours ago"
# first of next month
(today + 1 mo).with({day: 1})
# → 2026-11-01
# months with a Friday the 13th
(1..=12).filter(|m| date(2026, m, 13).weekday == "Friday")
# → [2, 3, 11]
# from a Unix timestamp
date(1798178400)
# → 2026-12-25
```
