# Run: zil examples/demo.zil

# Units
trip = 42 km
print("trip: {trip to mi}, at 60 mph takes {trip / 60 mph to min}")
print("oven: {350 F to C}")
print("download: {4.7 GB / 50 Mbps to min}")

# Bases
mask = 0xff00 to bin
print("mask: {mask}, low byte: {0xab to dec}")

# Strings
title = "the quick brown fox"
print(title.split.map(|w| w.capitalize).join(" "))
print("digits in 'a1b22c333': {"a1b22c333".find_all(r"\d+")}")
print("slug: {title.replace(r"\s+", "-")}")
print("sha256: {"hello".sha256[..12]}...")

# Dates
days = date("2026-12-25") - now
print("days until Christmas: {round(days to d)}")
