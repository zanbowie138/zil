# fun.games

dice notation, the birthday paradox, odds in plain words, Western and Chinese zodiac

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`roll(dice?: str)`](#roll) | roll dice like "3d6+2", "d20", "4d6kh3" (keep highest 3) or "2d20kl1"; returns {total, rolls}. Default 1d6 |
| [`birthday_paradox(people: int, days?: int)`](#birthday_paradox) | the chance at least two of n people share a birthday (or any of `days` equally likely values) |
| [`odds(p: num)`](#odds) | a probability as "1 in N", next to a familiar event about as likely |
| [`zodiac(day?: date)`](#zodiac) | the Western zodiac sign for a date (default today) |
| [`chinese_zodiac(year: int\|date)`](#chinese_zodiac) | the Chinese zodiac element and animal; by Gregorian year, so January dates before Lunar New Year come out a year late |

### roll

`roll(dice?: str)`: roll dice like "3d6+2", "d20", "4d6kh3" (keep highest 3) or "2d20kl1"; returns {total, rolls}. Default 1d6

```zil
roll("3d6+2")
# → {total: 8, rolls: [1, 4, 1]}
roll("d%").total
# → 83
```

See also: [rand](../math/random.md#rand), [coin](../fun.md#coin)

### birthday_paradox

`birthday_paradox(people: int, days?: int)`: the chance at least two of n people share a birthday (or any of `days` equally likely values)

```zil
birthday_paradox(23)
# → 0.507297
birthday_paradox(70)
# → 0.99916
birthday_paradox(10, 100)
# → 0.371843
```

See also: [odds](../fun/games.md#odds), [choose](../math/numtheory.md#choose)

### odds

`odds(p: num)`: a probability as "1 in N", next to a familiar event about as likely

```zil
odds(1 / 1000)
# → "1 in 1,000: about as likely as flipping ten heads in a row (1 in 1,024)"
odds(0.5)
# → "1 in 2: about as likely as a coin landing heads (1 in 2)"
odds(2 ** -64)
# → "1 in 1.84 × 10^19: about as likely as a perfect March Madness bracket (1 in 9.20 × 10^18)"
```

See also: [birthday_paradox](../fun/games.md#birthday_paradox)

### zodiac

`zodiac(day?: date)`: the Western zodiac sign for a date (default today)

```zil
zodiac(date("2000-01-01"))
# → "♑ Capricorn"
```

See also: [chinese_zodiac](../fun/games.md#chinese_zodiac)

### chinese_zodiac

`chinese_zodiac(year: int|date)`: the Chinese zodiac element and animal; by Gregorian year, so January dates before Lunar New Year come out a year late

```zil
chinese_zodiac(2026)
# → "Fire Horse"
chinese_zodiac(1984)
# → "Wood Rat"
```

See also: [zodiac](../fun/games.md#zodiac)

## More examples

### games

```zil
# a D&D stat: 4d6, keep the best 3
roll("4d6kh3")
# → {total: 14, rolls: [5, 3, 6, 2]}
# attack with advantage
roll("2d20kh1+5").total
# → 17
# a party of 23
birthday_paradox(23)
# → 0.507297
# how unlikely is the jackpot?
odds(1 / 292201338)
# → "1 in 292,201,338: about as likely as winning the Powerball jackpot (1 in 292,201,338)"
# your sign
[zodiac(date("1990-08-10")), chinese_zodiac(1990)]
# → ["♌ Leo", "Metal Horse"]
```
