# time.calendar_math

periods, weekdays, business days, week numbers, month grids, cron schedules; weeks start Monday

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`weekday_num(d: date)`](#weekday_num) | weekday as a number, Monday = 1 ... Sunday = 7 |
| [`day_of_year(d: date)`](#day_of_year) | day of the year, 1-366 |
| [`iso_week(d: date)`](#iso_week) | ISO 8601 week number (weeks start Monday) |
| [`quarter(d: date)`](#quarter) | quarter of the year, 1-4 |
| [`leap_year(d: date\|int)`](#leap_year) | whether the year is a leap year |
| [`days_in_month(d: date)`](#days_in_month) | number of days in the date's month |
| [`days_in_year(d: date)`](#days_in_year) | 365 or 366 |
| [`start_of(d: date, period: str)`](#start_of) | start of the second/minute/hour/day/week/month/quarter/year |
| [`end_of(d: date, period: str)`](#end_of) | last moment of the period |
| [`next(d: date, weekday: str) / next(cron: str, n?: int)`](#next) | the next given weekday strictly after d; or the next run of a cron expression, or a list of the next n |
| [`prev(d: date, weekday: str)`](#prev) | the last given weekday strictly before d |
| [`nth_weekday(d: date, n: int, weekday: str)`](#nth_weekday) | nth weekday of d's month; negative counts from the end |
| [`add_workdays(d: date, n: int, holidays?: list\|str)`](#add_workdays) | move n Monday-Friday days, skipping holidays (a list of dates, or "US"/"UK"); negative goes back |
| [`workdays(a: date, b: date, holidays?: list\|str)`](#workdays) | Monday-Friday days from a up to (not including) b, minus holidays |
| [`holidays(year: int, country: str)`](#holidays) | public holidays as observed (moved off weekends): "US" federal or "UK" England bank holidays |
| [`cron(expr: str)`](#cron) | a cron expression (minute hour day month weekday, or @daily etc.) in plain English |
| [`calendar(d: date) / calendar(year: int, month: int)`](#calendar) | month grid, weeks starting Monday |

### weekday_num

`weekday_num(d: date)`: weekday as a number, Monday = 1 ... Sunday = 7

```zil
today.weekday_num
# → 3
```

See also: [weekday](../time.md#weekday)

### day_of_year

`day_of_year(d: date)`: day of the year, 1-366

```zil
date("2026-12-31").day_of_year
# → 365
```

See also: [iso_week](../time/calendar_math.md#iso_week)

### iso_week

`iso_week(d: date)`: ISO 8601 week number (weeks start Monday)

```zil
date("2026-12-31").iso_week
# → 53
```

See also: [day_of_year](../time/calendar_math.md#day_of_year), [quarter](../time/calendar_math.md#quarter)

### quarter

`quarter(d: date)`: quarter of the year, 1-4

```zil
today.quarter
# → 4
```

See also: [iso_week](../time/calendar_math.md#iso_week)

### leap_year

`leap_year(d: date|int)`: whether the year is a leap year

```zil
leap_year(2028)
# → true
today.leap_year
# → false
```

See also: [days_in_year](../time/calendar_math.md#days_in_year)

### days_in_month

`days_in_month(d: date)`: number of days in the date's month

```zil
date("2028-02-10").days_in_month
# → 29
```

See also: [days_in_year](../time/calendar_math.md#days_in_year)

### days_in_year

`days_in_year(d: date)`: 365 or 366

```zil
today.days_in_year
# → 365
```

See also: [leap_year](../time/calendar_math.md#leap_year)

### start_of

`start_of(d: date, period: str)`: start of the second/minute/hour/day/week/month/quarter/year

```zil
now.start_of("week")
# → 2026-10-05
now.start_of("quarter")
# → 2026-10-01
```

See also: [end_of](../time/calendar_math.md#end_of), [with](../time.md#with)

### end_of

`end_of(d: date, period: str)`: last moment of the period

```zil
now.end_of("month")
# → 2026-10-31 23:59:59 -05:00
(today.end_of("year") - now).parts
# → "85 d 10 h 1 min"
```

See also: [start_of](../time/calendar_math.md#start_of)

### next

`next(d: date, weekday: str) / next(cron: str, n?: int)`: the next given weekday strictly after d; or the next run of a cron expression, or a list of the next n

```zil
today.next("friday")
# → 2026-10-09
now.next("mon")
# → 2026-10-12 14:58:32 -05:00
"0 9 * * mon".next
# → 2026-10-12 09:00:00 -05:00
"@monthly".next(2)
# → [2026-11-01, 2026-12-01]
```

See also: [prev](../time/calendar_math.md#prev), [nth_weekday](../time/calendar_math.md#nth_weekday), [cron](../time/calendar_math.md#cron)

### prev

`prev(d: date, weekday: str)`: the last given weekday strictly before d

```zil
today.prev("sunday")
# → 2026-10-04
```

See also: [next](../time/calendar_math.md#next)

### nth_weekday

`nth_weekday(d: date, n: int, weekday: str)`: nth weekday of d's month; negative counts from the end

```zil
date(2026, 11, 1).nth_weekday(4, "thu")
# → 2026-11-26
date(2026, 5, 1).nth_weekday(-1, "mon")
# → 2026-05-25
```

See also: [next](../time/calendar_math.md#next)

### add_workdays

`add_workdays(d: date, n: int, holidays?: list|str)`: move n Monday-Friday days, skipping holidays (a list of dates, or "US"/"UK"); negative goes back

```zil
today.add_workdays(10)
# → 2026-10-21
date("2026-12-24").add_workdays(1, "US")
# → 2026-12-28
date("2026-12-24").add_workdays(1, [date("2026-12-28")])
# → 2026-12-25
```

See also: [workdays](../time/calendar_math.md#workdays), [holidays](../time/calendar_math.md#holidays), [is_weekend](../time.md#is_weekend)

### workdays

`workdays(a: date, b: date, holidays?: list|str)`: Monday-Friday days from a up to (not including) b, minus holidays

```zil
workdays(today, date("2026-12-25"))
# → 57
workdays(date("2026-12-01"), date("2027-01-01"), "UK")
# → 21
```

See also: [add_workdays](../time/calendar_math.md#add_workdays), [holidays](../time/calendar_math.md#holidays)

### holidays

`holidays(year: int, country: str)`: public holidays as observed (moved off weekends): "US" federal or "UK" England bank holidays

```zil
holidays(2026, "US")
# → [2026-01-01, 2026-01-19, 2026-02-16, 2026-05-25, 2026-06-19, 2026-07-03, 2026-09-07, 2026-10-12, 2026-11-11, 2026-11-26, 2026-12-25]
holidays(2026, "UK").len
# → 8
```

See also: [add_workdays](../time/calendar_math.md#add_workdays), [workdays](../time/calendar_math.md#workdays)

### cron

`cron(expr: str)`: a cron expression (minute hour day month weekday, or @daily etc.) in plain English

```zil
cron("0 9 * * 1-5")
# → "at 09:00, on Monday through Friday"
cron("30 4 1,15 * fri")
# → "at 04:30, on days 1 and 15 of the month, or on Friday"
cron("@hourly")
# → "at minute 0 of every hour"
```

See also: [next](../time/calendar_math.md#next)

### calendar

`calendar(d: date) / calendar(year: int, month: int)`: month grid, weeks starting Monday

```zil
calendar(2026, 12)
# → "   December 2026    \nMo Tu We Th Fr Sa Su\n    1  2  3  4  5  6\n 7  8  9 10 11 12 13\n14 15 16 17 18 19 20\n21 22 23 24 25 26 27\n28 29 30 31"
```

See also: [date](../time.md#date)

## More examples

### calendar_math

```zil
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
# skipping US holidays
date("2026-12-24").add_workdays(3, "US")
# → 2026-12-30
# ISO week number
today.iso_week
# → 41
# what does this cron line mean?
cron("*/15 9-17 * * mon-fri")
# → "every 15 minutes, during hours 9 through 17, on Monday through Friday"
# when does it run next?
"0 9 * * 1-5".next(3)
# → [2026-10-08 09:00:00 -05:00, 2026-10-09 09:00:00 -05:00, 2026-10-12 09:00:00 -05:00]
```
