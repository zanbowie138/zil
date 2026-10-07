# art.toys

ASCII Mandelbrot set, Conway's Game of Life, random mazes

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`mandelbrot(width?: int, height?: int)`](#mandelbrot) | the Mandelbrot set in ASCII, default 72×24 |
| [`life(start: str, steps?: int)`](#life) | run Conway's Game of Life for steps (default 1) and draw the live cells; start is rows of # and . or one of "glider", "blinker", "pulsar", "r_pentomino" |
| [`maze(width?: int, height?: int)`](#maze) | a random perfect maze (one path between any two cells), default 20×10 cells |

### mandelbrot

`mandelbrot(width?: int, height?: int)`: the Mandelbrot set in ASCII, default 72×24

```zil
mandelbrot(40, 12)
# → "\n                          ...:\n                       ...:@@@..\n                    ..#+=@@@@@@@:#:.\n            .........#@@@@@@@@@@@@-.\n           ..:@@@@@:@@@@@@@@@@@@@@@-\n   @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@-..\n           ..:@@@@@:@@@@@@@@@@@@@@@-\n            .........#@@@@@@@@@@@@-.\n                    ..#+=@@@@@@@:#:.\n                       ...:@@@..\n                          ...:"
```

See also: [life](../art/toys.md#life)

### life

`life(start: str, steps?: int)`: run Conway's Game of Life for steps (default 1) and draw the live cells; start is rows of # and . or one of "glider", "blinker", "pulsar", "r_pentomino"

```zil
life("blinker")
# → "#\n#\n#"
life("glider", 4)
# → ".#.\n..#\n###"
```

See also: [maze](../art/toys.md#maze)

### maze

`maze(width?: int, height?: int)`: a random perfect maze (one path between any two cells), default 20×10 cells

```zil
maze(6, 3)
# → "██  ██████████████████████\n██      ██              ██\n██████  ██████  ██████  ██\n██  ██  ██      ██  ██  ██\n██  ██  ██  ██████  ██  ██\n██          ██          ██\n██████████████████████  ██"
```

See also: [life](../art/toys.md#life)

## More examples

### toys

```zil
# the Mandelbrot set
mandelbrot()
# → "\n                                               .-.\n                                               .......\n                                              ...-=:...\n                                          .....:@@@@@:..\n                                      .........:@@@@+:.........\n                                   ....:@@:%@@@@@@@@@@@@@:.:::#.\n                                .....*.#@@@@@@@@@@@@@@@@@@@@@:..\n                     ................-@@@@@@@@@@@@@@@@@@@@@@@=...\n                    ....:@-.:@::...:=@@@@@@@@@@@@@@@@@@@@@@@@@-..\n                   ...:.-@@@@@@@@=:-@@@@@@@@@@@@@@@@@@@@@@@@@@@-.\n            ........#=:+@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@.\n     @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@+:...\n            ........#=:+@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@.\n                   ...:.-@@@@@@@@=:-@@@@@@@@@@@@@@@@@@@@@@@@@@@-.\n                    ....:@-.:@::...:=@@@@@@@@@@@@@@@@@@@@@@@@@-..\n                     ................-@@@@@@@@@@@@@@@@@@@@@@@=...\n                                .....*.#@@@@@@@@@@@@@@@@@@@@@:..\n                                   ....:@@:%@@@@@@@@@@@@@:.:::#.\n                                      .........:@@@@+:.........\n                                          .....:@@@@@:..\n                                              ...-=:...\n                                               .......\n                                               .-."
# a glider, 8 steps on
life("glider", 8)
# → ".#.\n..#\n###"
# draw your own
life(".#.\n.#.\n.#.", 1)
# → "###"
# a maze
maze(12, 6)
# → "██  ██████████████████████████████████████████████\n██      ██                  ██                  ██\n██████  ██████████████  ██  ██████████  ██████  ██\n██  ██          ██      ██              ██      ██\n██  ██████████  ██  ██████████████████████  ██████\n██  ██          ██          ██      ██      ██  ██\n██  ██  ██████████████████  ██  ██  ██  ██████  ██\n██      ██                  ██  ██  ██  ██      ██\n██  ██████  ██████████████████████  ██  ██  ██████\n██  ██      ██                      ██  ██      ██\n██  ██  ██████  ██████  ██████████████  ██████  ██\n██      ██          ██                          ██\n██████████████████████████████████████████████  ██"
```
