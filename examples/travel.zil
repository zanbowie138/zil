# Run: zil examples/travel.zil
# Cities, distances, time zones and the sky.

for [a, b] in [["London", "New York"], ["Sydney", "Santiago"], ["Cairo", "Tokyo"]] {
  d = great_circle(a, b)
  print("{a} -> {b}: {round(d)}, about {(d / 880 km/h).parts} in the air")
}

jp = country("Japan")
print("{jp.flag} {jp.name}: capital {jp.capital}, currency {jp.currency}")
print("five biggest cities: {cities().take(5).map(|c| c.name)}")

# Daylight on the June solstice, south to north
for place in ["Singapore", "Cairo", "Paris", "Oslo", "Reykjavik"] {
  print("  {place.pad(10)} {day_length(place, date(2026, 6, 21)).parts}")
}

print("moon tonight: {moon_phase()}")
