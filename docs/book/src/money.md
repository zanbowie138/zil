# money

currencies shown as money, pay, interest, loans, splitting bills

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

### types

```zil
# money
1234.5 USD
# → $1,234.50
```

### literals

```zil
# $ € £ before a number
$25/h
# → $25.00/h
# any currency code
1200 JPY
# → ¥1,200
```

### paid time

```zil
# workday 8 h, workwk 40 h, workmo, workyr 2080 h; USD/yr stays calendar time
# hourly to salary
$25/h to USD/workyr
# → $52,000.00/workyr
# salary to hourly
85000 USD/workyr to USD/h
# → $40.87/h
```

### arguments

```zil
# functions tell arguments apart by unit: $1000 a sum, $500/mo a payment,
# 7%/yr a rate, 10 yr a time, "monthly" how often interest compounds
```

### currencies

```zil
# rates load (with a prompt) only to convert between currencies: 20 USD to EUR
```

## Functions

| function | description |
|---|---|
| [`salary(rate, opts?)`](#salary) | pay per hour, day, week, month and year; opts {hours: 40 a week, weeks: 52 paid} |
| [`grow(sum, rate, time, payment?, compounding?)`](#grow) | future value: a sum and/or payments growing at a rate |
| [`cagr(start, end, time)`](#cagr) | compound annual growth rate |
| [`doubling(rate, compounding?)`](#doubling) | time for money to double |
| [`apy(rate, compounding)`](#apy) | effective yearly rate after compounding |
| [`real_rate(rate, inflation)`](#real_rate) | return after inflation |
| [`payment(loan, rate, time, compounding?)`](#payment) | monthly payment that pays off a loan |
| [`payoff(balance, rate, payment, compounding?)`](#payoff) | how long a payment takes to clear a balance, in whole payments |
| [`amortize(loan, rate, time, opts?)`](#amortize) | payment schedule: {n, payment, interest, principal, balance} per month; opts {extra: $200/mo} |
| [`share(sum, n or weights)`](#share) | split into parts that add up exactly, to the cent |
| [`settle({name: paid, ...})`](#settle) | who pays whom so everyone paid the same |
| [`change(from, to)`](#change) | fractional change from one value to another |
| [`margin(cost, price)`](#margin) | profit as a fraction of the price |
| [`markup(cost, price)`](#markup) | profit as a fraction of the cost |

### salary

`salary(rate, opts?)`: pay per hour, day, week, month and year; opts {hours: 40 a week, weeks: 52 paid}

```zil
salary($25/h)
# → {hour: $25.00, day: $200.00, week: $1,000.00, month: $4,333.33, year: $52,000.00}
salary(85000 USD/yr, {hours: 37.5})
# → {hour: $43.59, day: $326.92, week: $1,634.62, month: $7,083.33, year: $85,000.00}
```

See also: [grow](money.md#grow)

### grow

`grow(sum, rate, time, payment?, compounding?)`: future value: a sum and/or payments growing at a rate

```zil
grow($10000, 7%/yr, 30 yr)
# → $76,122.55
grow($0, 7%/yr, 30 yr, $500/mo)
# → $609,985.50
grow($1000, 5%/yr, 1 yr, "daily")
# → $1,051.27
```

See also: [cagr](money.md#cagr), [payment](money.md#payment)

### cagr

`cagr(start, end, time)`: compound annual growth rate

```zil
cagr($1000, $2500, 8 yr)
# → 12.1353%/yr
```

See also: [grow](money.md#grow), [change](money.md#change)

### doubling

`doubling(rate, compounding?)`: time for money to double

```zil
doubling(7%/yr)
# → 10.2448 yr
```

See also: [grow](money.md#grow)

### apy

`apy(rate, compounding)`: effective yearly rate after compounding

```zil
apy(5%/yr, "daily")
# → 5.12675%/yr
apy(5%/yr, "monthly")
# → 5.11619%/yr
```

See also: [grow](money.md#grow)

### real_rate

`real_rate(rate, inflation)`: return after inflation

```zil
real_rate(7%/yr, 3%/yr)
# → 3.8835%/yr
```

See also: [grow](money.md#grow)

### payment

`payment(loan, rate, time, compounding?)`: monthly payment that pays off a loan

```zil
payment($400000, 6.5%/yr, 30 yr)
# → $2,528.27/mo
payment($25000, 7%/yr, 5 yr) * 5 yr
# → $29,701.80
```

See also: [payoff](money.md#payoff), [amortize](money.md#amortize)

### payoff

`payoff(balance, rate, payment, compounding?)`: how long a payment takes to clear a balance, in whole payments

```zil
payoff($5000, 22%/yr, $200/mo)
# → 34 mo
today + payoff($5000, 22%/yr, $200/mo)
# → 2029-08-06
```

See also: [payment](money.md#payment)

### amortize

`amortize(loan, rate, time, opts?)`: payment schedule: {n, payment, interest, principal, balance} per month; opts {extra: $200/mo}

```zil
amortize($1000, 12%/yr, 3 mo)
# → [{n: 1, payment: $340.02, interest: $10.00, principal: $330.02, balance: $669.98}, {n: 2, payment: $340.02, interest: $6.70, principal: $333.32, balance: $336.66}, {n: 3, payment: $340.02, interest: $3.37, principal: $336.66, balance: $0.00}]
amortize($400000, 6.5%/yr, 30 yr, {extra: $300/mo}).len * 1 mo to yr
# → 22.4167 yr
```

See also: [payment](money.md#payment)

### share

`share(sum, n or weights)`: split into parts that add up exactly, to the cent

```zil
share($100, 3)
# → [$33.34, $33.33, $33.33]
share($100, [2, 1, 1])
# → [$50.00, $25.00, $25.00]
```

See also: [settle](money.md#settle)

### settle

`settle({name: paid, ...})`: who pays whom so everyone paid the same

```zil
settle({ana: $120, ben: $0, cy: $30})
# → ["ben pays ana $50.00", "cy pays ana $20.00"]
```

See also: [share](money.md#share)

### change

`change(from, to)`: fractional change from one value to another

```zil
change($80, $100)
# → 0.25
change(80, 60).percent
# → "-25%"
```

See also: [margin](money.md#margin), [cagr](money.md#cagr)

### margin

`margin(cost, price)`: profit as a fraction of the price

```zil
margin($60, $100)
# → 0.4
```

See also: [markup](money.md#markup)

### markup

`markup(cost, price)`: profit as a fraction of the cost

```zil
markup($60, $100)
# → 0.666667
```

See also: [margin](money.md#margin)

## More examples

### money

```zil
# yearly salary from hourly
$25/h to USD/workyr
# → $52,000.00/workyr
# what a meeting costs
6 * 95000 USD/workyr * 1 h
# → $274.04
# hours of work to buy it
$1200 / ($40/h) to workday
# → 3.75 workday
# unit price
$4.99 / 12 oz to USD/lb
# → $6.65/lb
# a subscription per year
$15.99/mo to USD/yr
# → $191.88/yr
# mortgage payment
payment($400000, 6.5%/yr, 30 yr)
# → $2,528.27/mo
# interest over the loan
payment($400000, 6.5%/yr, 30 yr) * 30 yr - $400000
# → $510,177.95
# debt-free date
today + payoff($5000, 22%/yr, $200/mo)
# → 2029-08-06
# saving $500 a month
grow($0, 7%/yr, 30 yr, $500/mo)
# → $609,985.50
# split a bill
share($100, 3)
# → [$33.34, $33.33, $33.33]
# who owes whom
settle({ana: $120, ben: $0, cy: $30})
# → ["ben pays ana $50.00", "cy pays ana $20.00"]
```
