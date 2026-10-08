# Run: zil examples/units.zil
# Quantities carry their units through every step.

# Road trip: how much gas, and what does it cost?
distance = 640 km
mileage = 32 mi / 1 gal
fuel = distance / mileage to gal
print("fuel needed: {fuel} ({fuel to L})")
print("at $3.80/gal: {fuel * $3.80 / 1 gal}")

# Is the network faster than walking a hard drive across town?
drive = 2 TB
walk = 3 km / 5 kph
print("2 TB over 100 Mbps: {drive / 100 Mbps to h}")
print("2 TB on foot: {drive / walk to Mbps}")

# Physics, the easy way
work = 70 kg * 9.81 m/s^2 * 8848 m
print("climbing Everest takes {work to kcal}, or {work to kWh}")

# One value, several units
print("6.2 ft is {6.2 ft to ft in}")
print("100000 s is {100000 s to d h min}")

# Measurements with error bars
side = 12.0 cm ± 0.1 cm
print("cube volume: {side ** 3 to L}")
