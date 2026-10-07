# art.motion

play a list of frames, or a function of the frame number, as an animation in the terminal

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### animate

```zil
# a glider crossing the screen: animate(|i| life("glider", i), 8, 40)
# a fractal growing: animate(|i| fractal("tree", i + 1, 40), 2, 7)
```

## Functions

| function | description |
|---|---|
| [`animate(frames: list\|fn, fps?: num, count?: int)`](#animate) | draw frames one after another in place, fps a second (default 10). A list plays count times (default once); a function gets the frame number from 0 and plays count frames (default 60). Off a terminal only the last frame prints |

### animate

`animate(frames: list|fn, fps?: num, count?: int)`: draw frames one after another in place, fps a second (default 10). A list plays count times (default once); a function gets the frame number from 0 and plays count frames (default 60). Off a terminal only the last frame prints

```zil
animate(|i| life("glider", i), 8, 40)
animate(["|", "/", "-", "\\"], 12, 5)
```

See also: [life](../art/toys.md#life), [fractal](../art/generate.md#fractal)
