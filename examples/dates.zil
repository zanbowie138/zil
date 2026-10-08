# Run: zil examples/dates.zil

launch = date("2027-03-15 09:00")
print("launch is {launch.relative}, on a {launch.weekday}")
print("work days until then: {workdays(today, launch)}")

# Every Friday the 13th for the next few years
fri13 = (2026..2031).map(|y| (1..=12).map(|m| date(y, m, 13))).flatten.filter(|d| d.weekday == "Friday")
print("Friday the 13ths: {fri13.map(|d| d.format("%b %Y"))}")

# Holidays that move around
print("Thanksgiving 2026: {date(2026, 11, 1).nth_weekday(4, "thu").format("%B %d")}")
print("Memorial Day 2026: {date(2026, 5, 1).nth_weekday(-1, "mon").format("%B %d")}")

# One meeting, three time zones
meeting = date("2026-11-02T16:00Z")
for city in ["Los Angeles", "London", "Mumbai"] {
  print("  {city.pad(12)} {(meeting to (city)).format("%a %H:%M")}")
}

print("the unix epoch was {diff(date(0), now)} ago")

print(calendar(2026, 12))
