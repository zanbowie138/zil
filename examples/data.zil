# Run: zil examples/data.zil
# CSV in, tables and charts out.

csv = "city,pop_m,area_km2
Tokyo,37.4,2194
Delhi,32.9,1484
Shanghai,29.2,6341
Sao Paulo,22.4,1521
Mexico City,21.8,1485
Cairo,21.3,3085"

rows = csv.from_csv.map(|r| {city: r.city, pop_m: r.pop_m, per_km2: round(r.pop_m * 1e6 / r.area_km2)})
print(pretty(rows.table.sort_by_desc("per_km2")))
print(rows.map(|r| [r.city, r.pop_m]).from_entries.bars)

temps = [3, 5, 9, 14, 18, 22, 25, 24, 20, 14, 8, 4]
print("monthly temps: {temps.sparkline}")
print("mean {round(temps.avg, 1)}, median {temps.median}, 90th percentile {percentile(temps, 90)}")

# Roll two dice a thousand times
rolls = (0..1000).map(|_| rand(1, 6) + rand(1, 6))
print((2..=12).map(|n| [n, rolls.count(n)]).from_entries.bars)
