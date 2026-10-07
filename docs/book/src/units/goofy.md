# units.goofy

bananas for scale, smoots, fortnights; every item also works as item_for_scale

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

## Units

| kind | names |
|---|---|
| length | banana, bananas; credit_card, credit_cards; attoparsec, attoparsecs; beard_second, beard_seconds; smoot, smoots; altuve, altuves; cubit, cubits; hand, hands; giraffe, giraffes; school_bus, school_buses; blue_whale, blue_whales; football_field, football_fields; statue_of_liberty; furlong, furlongs; eiffel_tower, eiffel_towers; sheppey, sheppeys; league, leagues; everest, everests; marathon, marathons |
| mass | paperclip, paperclips; bowling_ball, bowling_balls; corgi, corgis; slug, slugs; firkin, firkins; grand_piano, grand_pianos; honda_civic, honda_civics; elephant, elephants |
| time | shake, shakes; jiffy, jiffies; nanocentury, nanocenturies; moment, moments; microcentury, microcenturies; scaramucci, scaramuccis; fortnight, fortnights; dog_year, dog_years; friedman, friedmans |
| volume | barrel, barrels, bbl; hogshead, hogsheads; bathtub, bathtubs; olympic_pool, olympic_pools |
| area | barn, barns; parking_space, parking_spaces; tennis_court, tennis_courts; rhode_island; wales |
| speed | snail, snails; mach |
| data | nibble, nibbles; floppy, floppies; cdrom, cdroms; dvd, dvds; library_of_congress |
| energy | big_mac, big_macs; tnt, ton_tnt; hiroshima, hiroshimas |
| power | donkeypower; toaster, toasters |

## Functions

| function | description |
|---|---|
| [`for_scale(qty)`](#for_scale) | the quantity in whichever goofy unit gives the most relatable count |

### for_scale

`for_scale(qty)`: the quantity in whichever goofy unit gives the most relatable count

```zil
1.8 m.for_scale
# → 3.93701 cubit
70 kg.for_scale
# → 4.79652 slug
2 h.for_scale
# → 2.28154 microcentury
```

## More examples

### goofy

```zil
# height in bananas
1.8 m to banana_for_scale
# → 10.1124 banana
# the classic speed unit
1 mph to furlong/fortnight
# → 2688 furlong/fortnight
# Harvard Bridge
364.4 smoot to m
# → 620.136 m
# pick a fitting item
8848 m.for_scale
# → 1.83263 league
# a lecture
50 min to microcentury
# → 0.950643 microcentury
# lunch energy
1 big_mac to kWh
# → 0.654331 kWh
```
