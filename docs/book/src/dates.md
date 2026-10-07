# dates

parsing, calendar math, relative text, month grids; weeks start Monday

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
# to UTC, to local
date("2026-12-25 18:30") to UTC
# → 2026-12-26 00:30:00 +00:00
# to "Zone/Name"
date("2026-12-25 18:30") to "Asia/Tokyo"
# → 2026-12-26 09:30:00 +09:00
```

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
| [`weekday_num(d)`](#weekday_num) | weekday as a number, Monday = 1 ... Sunday = 7 |
| [`day_of_year(d)`](#day_of_year) | day of the year, 1-366 |
| [`iso_week(d)`](#iso_week) | ISO 8601 week number (weeks start Monday) |
| [`quarter(d)`](#quarter) | quarter of the year, 1-4 |
| [`leap_year(d or year)`](#leap_year) | whether the year is a leap year |
| [`days_in_month(d)`](#days_in_month) | number of days in the date's month |
| [`days_in_year(d)`](#days_in_year) | 365 or 366 |
| [`is_weekend(d)`](#is_weekend) | Saturday or Sunday |
| [`is_weekday(d)`](#is_weekday) | Monday through Friday |
| [`is_today(d)`](#is_today) | same calendar day as now |
| [`is_past(d)`](#is_past) | before now |
| [`is_future(d)`](#is_future) | after now |
| [`start_of(d, period)`](#start_of) | start of the second/minute/hour/day/week/month/quarter/year |
| [`end_of(d, period)`](#end_of) | last moment of the period |
| [`next(d, weekday)`](#next) | the next given weekday strictly after d |
| [`prev(d, weekday)`](#prev) | the last given weekday strictly before d |
| [`nth_weekday(d, n, weekday)`](#nth_weekday) | nth weekday of d's month; negative counts from the end |
| [`add_workdays(d, n)`](#add_workdays) | move n Monday-Friday days (no holidays); negative goes back |
| [`workdays(a, b)`](#workdays) | Monday-Friday days from a up to (not including) b |
| [`age(d)`](#age) | whole years since d |
| [`diff(a, b)`](#diff) | calendar difference from a to b |
| [`relative(d)`](#relative) | "in 3 days", "2 hours ago" |
| [`parts(duration)`](#parts) | duration in up to three of d, h, min, s; or `to d h min` for chosen units |
| [`calendar(d) / calendar(year, month)`](#calendar) | month grid, weeks starting Monday |
| [`unix(d)`](#unix) | seconds since 1970-01-01 UTC; same as `d to unix` |
| [`utc(d)`](#utc) | the same moment in UTC; same as `d to UTC` (`d to "Asia/Tokyo"` for any zone) |
| [`local(d)`](#local) | the same moment in the system time zone; same as `d to local` |

### date

`date(s) / date(s, fmt) / date(y, m, d, h?, min?, s?) / date(unix)`: parse ISO, US (12/25/2026), written (Dec 25 2026) or natural language dates

```zil
date("2026-12-25")
# → 2026-12-25
date("next friday at 5pm")
# → 2026-10-09 17:00:00 -05:00
date("3 days ago")
# → 2026-10-03 22:40:54 -05:00
date("25.12.2026", "%d.%m.%Y")
# → 2026-12-25
date(2026, 12, 25, 18, 30)
# → 2026-12-25 18:30:00 -06:00
date(0)
# → 1969-12-31 18:00:00 -06:00
```

See also: [format](dates.md#format), [with](dates.md#with), [start_of](dates.md#start_of)

### year

`year(d)`: year of a date

```zil
now.year
# → 2026
```

See also: [month](dates.md#month), [day](dates.md#day)

### month

`month(d)`: month of a date (1-12)

```zil
now.month
# → 10
```

See also: [year](dates.md#year), [day](dates.md#day)

### day

`day(d)`: day of the month

```zil
now.day
# → 6
```

See also: [month](dates.md#month), [weekday](dates.md#weekday)

### hour

`hour(d)`: hour of a date

```zil
now.hour
# → 22
```

See also: [minute](dates.md#minute), [second](dates.md#second)

### minute

`minute(d)`: minute of a date

```zil
now.minute
# → 40
```

See also: [hour](dates.md#hour), [second](dates.md#second)

### second

`second(d)`: second of a date

```zil
now.second
# → 54
```

See also: [hour](dates.md#hour), [minute](dates.md#minute)

### weekday

`weekday(d)`: day name

```zil
date("2026-12-25").weekday
# → "Friday"
```

See also: [day](dates.md#day), [format](dates.md#format)

### format

`format(d, fmt) / format(fmt, args...)`: format a date with strftime codes, or numbers printf-style

```zil
now.format("%B %d, %Y")
# → "October 06, 2026"
now.format("%H:%M")
# → "22:40"
format("%5.2f%%", 12.345)
# → "12.35%"
```

See also: [date](dates.md#date), [fixed](math.md#fixed)

### with

`with(d, fields)`: same date with fields replaced: year month day hour minute second

```zil
today.with({day: 1})
# → 2026-10-01
now.with({hour: 9, minute: 0})
# → 2026-10-06 09:00:54 -05:00
```

See also: [start_of](dates.md#start_of), [date](dates.md#date)

### weekday_num

`weekday_num(d)`: weekday as a number, Monday = 1 ... Sunday = 7

```zil
today.weekday_num
# → 2
```

See also: [weekday](dates.md#weekday)

### day_of_year

`day_of_year(d)`: day of the year, 1-366

```zil
date("2026-12-31").day_of_year
# → 365
```

See also: [iso_week](dates.md#iso_week)

### iso_week

`iso_week(d)`: ISO 8601 week number (weeks start Monday)

```zil
date("2026-12-31").iso_week
# → 53
```

See also: [day_of_year](dates.md#day_of_year), [quarter](dates.md#quarter)

### quarter

`quarter(d)`: quarter of the year, 1-4

```zil
today.quarter
# → 4
```

See also: [iso_week](dates.md#iso_week)

### leap_year

`leap_year(d or year)`: whether the year is a leap year

```zil
leap_year(2028)
# → true
today.leap_year
# → false
```

See also: [days_in_year](dates.md#days_in_year)

### days_in_month

`days_in_month(d)`: number of days in the date's month

```zil
date("2028-02-10").days_in_month
# → 29
```

See also: [days_in_year](dates.md#days_in_year)

### days_in_year

`days_in_year(d)`: 365 or 366

```zil
today.days_in_year
# → 365
```

See also: [leap_year](dates.md#leap_year)

### is_weekend

`is_weekend(d)`: Saturday or Sunday

```zil
date("2026-10-10").is_weekend
# → true
```

See also: [is_weekday](dates.md#is_weekday), [add_workdays](dates.md#add_workdays)

### is_weekday

`is_weekday(d)`: Monday through Friday

```zil
today.is_weekday
# → true
```

See also: [is_weekend](dates.md#is_weekend)

### is_today

`is_today(d)`: same calendar day as now

```zil
now.is_today
# → true
tomorrow.is_today
# → false
```

See also: [is_past](dates.md#is_past)

### is_past

`is_past(d)`: before now

```zil
yesterday.is_past
# → true
```

See also: [is_future](dates.md#is_future), [is_today](dates.md#is_today)

### is_future

`is_future(d)`: after now

```zil
tomorrow.is_future
# → true
```

See also: [is_past](dates.md#is_past)

### start_of

`start_of(d, period)`: start of the second/minute/hour/day/week/month/quarter/year

```zil
now.start_of("week")
# → 2026-10-05
now.start_of("quarter")
# → 2026-10-01
```

See also: [end_of](dates.md#end_of), [with](dates.md#with)

### end_of

`end_of(d, period)`: last moment of the period

```zil
now.end_of("month")
# → 2026-10-31 23:59:59 -05:00
(today.end_of("year") - now).parts
# → "86 d 2 h 19 min"
```

See also: [start_of](dates.md#start_of)

### next

`next(d, weekday)`: the next given weekday strictly after d

```zil
today.next("friday")
# → 2026-10-09
now.next("mon")
# → 2026-10-12 22:40:54 -05:00
```

See also: [prev](dates.md#prev), [nth_weekday](dates.md#nth_weekday)

### prev

`prev(d, weekday)`: the last given weekday strictly before d

```zil
today.prev("sunday")
# → 2026-10-04
```

See also: [next](dates.md#next)

### nth_weekday

`nth_weekday(d, n, weekday)`: nth weekday of d's month; negative counts from the end

```zil
date(2026, 11, 1).nth_weekday(4, "thu")
# → 2026-11-26
date(2026, 5, 1).nth_weekday(-1, "mon")
# → 2026-05-25
```

See also: [next](dates.md#next)

### add_workdays

`add_workdays(d, n)`: move n Monday-Friday days (no holidays); negative goes back

```zil
today.add_workdays(10)
# → 2026-10-20
```

See also: [workdays](dates.md#workdays), [is_weekend](dates.md#is_weekend)

### workdays

`workdays(a, b)`: Monday-Friday days from a up to (not including) b

```zil
workdays(today, date("2026-12-25"))
# → 58
```

See also: [add_workdays](dates.md#add_workdays)

### age

`age(d)`: whole years since d

```zil
date("1990-06-15").age
# → 36
```

See also: [diff](dates.md#diff)

### diff

`diff(a, b)`: calendar difference from a to b

```zil
diff(date("2025-08-03"), date("2026-10-06 04:00"))
# → "1 yr 2 mo 3 d 4 h"
```

See also: [age](dates.md#age), [relative](dates.md#relative), [parts](dates.md#parts)

### relative

`relative(d)`: "in 3 days", "2 hours ago"

```zil
date("2026-12-25").relative
# → "in 3 months"
(now - 3 h).relative
# → "3 hours ago"
```

See also: [diff](dates.md#diff), [parts](dates.md#parts)

### parts

`parts(duration)`: duration in up to three of d, h, min, s; or `to d h min` for chosen units

```zil
(date("2026-12-25") - date("2026-10-06 14:24")).parts
# → "79 d 10 h 36 min"
5000 s.parts
# → "1 h 23 min 20 s"
```

See also: [relative](dates.md#relative), [diff](dates.md#diff)

### calendar

`calendar(d) / calendar(year, month)`: month grid, weeks starting Monday

```zil
calendar(2026, 12)
# → "   December 2026    \nMo Tu We Th Fr Sa Su\n    1  2  3  4  5  6\n 7  8  9 10 11 12 13\n14 15 16 17 18 19 20\n21 22 23 24 25 26 27\n28 29 30 31"
```

See also: [date](dates.md#date)

### unix

`unix(d)`: seconds since 1970-01-01 UTC; same as `d to unix`

```zil
date(0).unix
# → 0
date(86400) to unix
# → 86400
```

See also: [date](dates.md#date)

### utc

`utc(d)`: the same moment in UTC; same as `d to UTC` (`d to "Asia/Tokyo"` for any zone)

```zil
date(0) to UTC
# → 1970-01-01 00:00:00 +00:00
date(0).utc.year
# → 1970
```

See also: [local](dates.md#local), [date](dates.md#date)

### local

`local(d)`: the same moment in the system time zone; same as `d to local`

```zil
(date(0) to UTC).local.year
# → 1969
```

See also: [utc](dates.md#utc)

## More examples

### dates

```zil
# day of the week
date("2026-12-25").weekday
# → "Friday"
# month math clamps to the end
date("2026-01-31") + 1 mo
# → 2026-02-28
# Thanksgiving: 4th Thursday
date(2026, 11, 1).nth_weekday(4, "thu")
# → 2026-11-26
# Memorial Day: last Monday
date(2026, 5, 1).nth_weekday(-1, "mon")
# → 2026-05-25
# 3 business days later
date("2026-12-24").add_workdays(3)
# → 2026-12-29
# workdays in December
workdays(date("2026-12-01"), date("2026-12-31"))
# → 22
# plain-English dates
date("next friday")
# → 2026-10-09
# time zones
date("2026-12-25 18:30") to "Asia/Tokyo"
# → 2026-12-26 09:30:00 +09:00
# countdown
(date("2027-01-01") - now).parts
# → "86 d 2 h 19 min"
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
# ISO week number
today.iso_week
# → 41
```
