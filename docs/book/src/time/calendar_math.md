# time.calendar_math

periods, weekdays, business days, week numbers, month grids; weeks start Monday

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`weekday_num(d)`](#weekday_num) | weekday as a number, Monday = 1 ... Sunday = 7 |
| [`day_of_year(d)`](#day_of_year) | day of the year, 1-366 |
| [`iso_week(d)`](#iso_week) | ISO 8601 week number (weeks start Monday) |
| [`quarter(d)`](#quarter) | quarter of the year, 1-4 |
| [`leap_year(d or year)`](#leap_year) | whether the year is a leap year |
| [`days_in_month(d)`](#days_in_month) | number of days in the date's month |
| [`days_in_year(d)`](#days_in_year) | 365 or 366 |
| [`start_of(d, period)`](#start_of) | start of the second/minute/hour/day/week/month/quarter/year |
| [`end_of(d, period)`](#end_of) | last moment of the period |
| [`next(d, weekday)`](#next) | the next given weekday strictly after d |
| [`prev(d, weekday)`](#prev) | the last given weekday strictly before d |
| [`nth_weekday(d, n, weekday)`](#nth_weekday) | nth weekday of d's month; negative counts from the end |
| [`add_workdays(d, n)`](#add_workdays) | move n Monday-Friday days (no holidays); negative goes back |
| [`workdays(a, b)`](#workdays) | Monday-Friday days from a up to (not including) b |
| [`calendar(d) / calendar(year, month)`](#calendar) | month grid, weeks starting Monday |

### weekday_num

`weekday_num(d)`: weekday as a number, Monday = 1 ... Sunday = 7

```zil
today.weekday_num
# → 2
```

See also: [weekday](../time.md#weekday)

### day_of_year

`day_of_year(d)`: day of the year, 1-366

```zil
date("2026-12-31").day_of_year
# → 365
```

See also: [iso_week](../time/calendar_math.md#iso_week)

### iso_week

`iso_week(d)`: ISO 8601 week number (weeks start Monday)

```zil
date("2026-12-31").iso_week
# → 53
```

See also: [day_of_year](../time/calendar_math.md#day_of_year), [quarter](../time/calendar_math.md#quarter)

### quarter

`quarter(d)`: quarter of the year, 1-4

```zil
today.quarter
# → 4
```

See also: [iso_week](../time/calendar_math.md#iso_week)

### leap_year

`leap_year(d or year)`: whether the year is a leap year

```zil
leap_year(2028)
# → true
today.leap_year
# → false
```

See also: [days_in_year](../time/calendar_math.md#days_in_year)

### days_in_month

`days_in_month(d)`: number of days in the date's month

```zil
date("2028-02-10").days_in_month
# → 29
```

See also: [days_in_year](../time/calendar_math.md#days_in_year)

### days_in_year

`days_in_year(d)`: 365 or 366

```zil
today.days_in_year
# → 365
```

See also: [leap_year](../time/calendar_math.md#leap_year)

### start_of

`start_of(d, period)`: start of the second/minute/hour/day/week/month/quarter/year

```zil
now.start_of("week")
# → 2026-10-05
now.start_of("quarter")
# → 2026-10-01
```

See also: [end_of](../time/calendar_math.md#end_of), [with](../time.md#with)

### end_of

`end_of(d, period)`: last moment of the period

```zil
now.end_of("month")
# → 2026-10-31 23:59:59 -05:00
(today.end_of("year") - now).parts
# → "86 d 1 h 29 min"
```

See also: [start_of](../time/calendar_math.md#start_of)

### next

`next(d, weekday)`: the next given weekday strictly after d

```zil
today.next("friday")
# → 2026-10-09
now.next("mon")
# → 2026-10-12 23:30:59 -05:00
```

See also: [prev](../time/calendar_math.md#prev), [nth_weekday](../time/calendar_math.md#nth_weekday)

### prev

`prev(d, weekday)`: the last given weekday strictly before d

```zil
today.prev("sunday")
# → 2026-10-04
```

See also: [next](../time/calendar_math.md#next)

### nth_weekday

`nth_weekday(d, n, weekday)`: nth weekday of d's month; negative counts from the end

```zil
date(2026, 11, 1).nth_weekday(4, "thu")
# → 2026-11-26
date(2026, 5, 1).nth_weekday(-1, "mon")
# → 2026-05-25
```

See also: [next](../time/calendar_math.md#next)

### add_workdays

`add_workdays(d, n)`: move n Monday-Friday days (no holidays); negative goes back

```zil
today.add_workdays(10)
# → 2026-10-20
```

See also: [workdays](../time/calendar_math.md#workdays), [is_weekend](../time.md#is_weekend)

### workdays

`workdays(a, b)`: Monday-Friday days from a up to (not including) b

```zil
workdays(today, date("2026-12-25"))
# → 58
```

See also: [add_workdays](../time/calendar_math.md#add_workdays)

### calendar

`calendar(d) / calendar(year, month)`: month grid, weeks starting Monday

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
# ISO week number
today.iso_week
# → 41
```
