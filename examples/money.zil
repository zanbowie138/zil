# Run: zil examples/money.zil
# Pay, loans and bill splitting. No network needed: nothing here mixes currencies.

wage = $32/h
print("yearly at 40 h/wk: {wage to USD/workyr}")
print("a $1,500 laptop costs {$1500 / wage to workday} of work")

# Mortgage
house = $450000
loan = house * 80%
monthly = payment(loan, 6.25%/yr, 30 yr)
print("monthly payment: {monthly}")
print("total interest: {monthly * 30 yr - loan}")

# Paying off a credit card
print("debt-free on: {today + payoff($4200, 21%/yr, $250/mo)}")

# Saving for retirement
print("$400/mo for 35 years at 7%: {grow($0, 7%/yr, 35 yr, $400/mo)}")
print("at 7%, money doubles every {doubling(7%/yr)}")

# Splitting a trip
paid = {ana: $412.60, ben: $95, cy: $0, dee: $188.40}
print("each person's share: {share(paid.values.sum, 4)[0]}")
for line in settle(paid) { print("  " + line) }
