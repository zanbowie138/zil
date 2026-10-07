# units.goofy

bananas for scale, smoots, fortnights; every item also works as item_for_scale

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Units

| kind | names |
|---|---|
| length | banana, bananas; credit_card, credit_cards; attoparsec, attoparsecs; beard_second, beard_seconds; human_hair, human_hairs; sheet_of_paper, sheets_of_paper; light_nanosecond, light_nanoseconds; smoot, smoots; altuve, altuves; cubit, cubits; hand, hands; step, steps; giraffe, giraffes; school_bus, school_buses; blue_whale, blue_whales; football_field, football_fields; city_block, city_blocks; statue_of_liberty; furlong, furlongs; eiffel_tower, eiffel_towers; sheppey, sheppeys; league, leagues; everest, everests; marathon, marathons |
| mass | grain_of_rice, grains_of_rice; paperclip, paperclips; us_penny, us_pennies; bowling_ball, bowling_balls; corgi, corgis; slug, slugs; firkin, firkins; grand_piano, grand_pianos; honda_civic, honda_civics; elephant, elephants; boeing_747, boeing_747s, jumbo_jet |
| time | planck_time, planck_times; shake, shakes; jiffy, jiffies; blink, blinks; nanocentury, nanocenturies; moment, moments; sol, sols; microcentury, microcenturies; scaramucci, scaramuccis; fortnight, fortnights; dog_year, dog_years; friedman, friedmans |
| volume | shot, shots; can, cans; wine_bottle, wine_bottles; barrel, barrels, bbl; hogshead, hogsheads; bathtub, bathtubs; olympic_pool, olympic_pools |
| area | barn, barns; parking_space, parking_spaces; tennis_court, tennis_courts; rhode_island; wales |
| speed | snail, snails; walking_pace; bike_pace; mach; lightspeed |
| data | nibble, nibbles; tweet, tweets; paragraph, paragraphs; novel, novels; floppy, floppies; photo, photos; cdrom, cdroms; human_genome, human_genomes; dvd, dvds; hour_of_hd_video, hours_of_hd_video; library_of_congress |
| energy | aa_battery, aa_batteries; banana_cal; big_mac, big_macs; gallon_gas, gallons_gas; tnt, ton_tnt; hiroshima, hiroshimas |
| power | human_resting; donkeypower; toaster, toasters |
| frequency | heartbeat, heartbeats; hummingbird_wingbeat, hummingbird_wingbeats |
| dose | bed, banana_dose |

## Functions

| function | description |
|---|---|
| [`for_scale(q: quantity)`](#for_scale) | the quantity in whichever goofy unit gives the most relatable count |

### for_scale

`for_scale(q: quantity)`: the quantity in whichever goofy unit gives the most relatable count

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
