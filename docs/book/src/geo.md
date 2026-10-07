# geo

countries and cities: lookups, distance, bearing, nearest city; place names work as time zones and currencies

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### names

```zil
# countries by name or ISO code
country("JPN").name
# → "Japan"
# short names and accents optional
[city("NYC").name, city("Sao Paulo").name]
# → ["New York City", "São Paulo"]
# the biggest city wins; add ", country" or ", state" to pick another
city("Hyderabad, PK").country
# → "Pakistan"
```

### places

```zil
# a place is a city or country name (a country means its capital), or a map with lat and lon
great_circle("Paris", {lat: 0, lon: 0})
# → 5436.93 km
```

### currencies

```zil
# $100 to "Vietnam" converts into a place's currency (rates load as for 20 USD to EUR)
```

### data

```zil
# GeoNames (CC-BY 4.0, geonames.org): capitals and cities over 500k people
```

## Functions

| function | description |
|---|---|
| [`country(name: str)`](#country) | a country by name or ISO code: {name, code, code3, capital, currency, zones, calling, population, area, continent, flag, languages, neighbours} |
| [`city(name: str)`](#city) | a city: {name, country, admin, population, lat, lon, zone}; the biggest of that name unless qualified like "Portland, US" |
| [`countries()`](#countries) | every country as a table |
| [`cities()`](#cities) | every city as a table, biggest first |
| [`great_circle(from: str\|map, to: str\|map)`](#great_circle) | great-circle distance between places |
| [`bearing(from: str\|map, to: str\|map)`](#bearing) | initial compass bearing from one place to another, clockwise from north |
| [`nearest(lat: num, lon: num)`](#nearest) | the known city closest to a point |

### country

`country(name: str)`: a country by name or ISO code: {name, code, code3, capital, currency, zones, calling, population, area, continent, flag, languages, neighbours}

```zil
country("Japan")
# → {name: "Japan", code: "JP", code3: "JPN", capital: "Tokyo", currency: "JPY", zones: ["Asia/Tokyo"], calling: "+81", population: 126529100, area: 377835 km^2, continent: "Asia", flag: "🇯🇵", languages: ["ja"], neighbours: []}
country("UK").currency
# → "GBP"
```

See also: [city](geo.md#city), [countries](geo.md#countries)

### city

`city(name: str)`: a city: {name, country, admin, population, lat, lon, zone}; the biggest of that name unless qualified like "Portland, US"

```zil
city("Tokyo")
# → {name: "Tokyo", country: "Japan", admin: "Tokyo", population: 9733276, lat: 35.6895, lon: 139.692, zone: "Asia/Tokyo"}
city("Seattle").zone
# → "America/Los_Angeles"
```

See also: [country](geo.md#country), [cities](geo.md#cities), [nearest](geo.md#nearest)

### countries

`countries()`: every country as a table

```zil
countries().len
# → 252
countries().filter(|c| c.continent == "Oceania").map(|c| c.name).take(3)
# → ["American Samoa", "Australia", "Cook Islands"]
```

See also: [country](geo.md#country), [cities](geo.md#cities)

### cities

`cities()`: every city as a table, biggest first

```zil
cities()[0].name
# → "Shanghai"
```

See also: [city](geo.md#city), [countries](geo.md#countries)

### great_circle

`great_circle(from: str|map, to: str|map)`: great-circle distance between places

```zil
great_circle("Tokyo", "Paris")
# → 9712.52 km
great_circle("NYC", "LA") to mi
# → 2445.56 mi
```

See also: [bearing](geo.md#bearing), [nearest](geo.md#nearest)

### bearing

`bearing(from: str|map, to: str|map)`: initial compass bearing from one place to another, clockwise from north

```zil
bearing("NYC", "London")
# → 51.2117 deg
```

See also: [great_circle](geo.md#great_circle)

### nearest

`nearest(lat: num, lon: num)`: the known city closest to a point

```zil
nearest(48.9, 2.3).name
# → "Paris"
```

See also: [city](geo.md#city), [great_circle](geo.md#great_circle)

## More examples

### geo

```zil
# currency of a country
country("Vietnam").currency
# → "VND"
# time in any city
date("2026-12-25 18:30") to "Seattle"
# → 2026-12-25 16:30:00 -08:00
# flight distance
great_circle("New York", "London") to mi
# → 3461.18 mi
# which way to Mecca
bearing("London", "Mecca")
# → 118.988 deg
# sunset in a city
sunset("Oslo", date(2026, 6, 21))
# → 2026-06-21 15:43:43 -05:00
# euro countries
countries().filter(|c| c.currency == "EUR").len
# → 36
# biggest cities in Japan
cities().filter(|c| c.country == "Japan").map(|c| c.name).take(3)
# → ["Tokyo", "Yokohama", "Osaka"]
```
