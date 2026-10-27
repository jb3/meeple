# Usage

## Running

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked      # or: brew install trunk
trunk serve --release
```

Open <http://localhost:8080> and click once to enable sound (browsers refuse audio
until a gesture). `--release` matters: the world generator is compute-heavy and the
dev profile is slow to load.

## Controls

| Input | Action |
| --- | --- |
| Drag / `W` `A` `S` `D` | Pan (the autopilot pauses; it resumes after 25 s) |
| Wheel / `+` `−` | Zoom |
| `Q` / `E` | Rotate |
| `Space` | Toggle autopilot |
| `P` | Cycle pixel scale (1 → 2 → 3 → 4 → 1) |
| `T` | Cycle time speed (1×, 6×, 30×, 120×) |
| `R` | Cycle weather (clear, cloudy, rain, storm, fog, snow) |
| `M` / `B` | Mute / music on-off |
| `F` | Fullscreen |
| `N` | New world (a random seed) |
| Click a villager | Show their card and follow them (`Esc` or click elsewhere to let go) |
| Click a train | Follow it for as long as it runs |
| `J` | Today's village newsletter (nearest town, with its station's comings and goings and the weather) |
| `Z` | "Where am I?" map of the land around the camera |
| `C` | The valley's chronicle: fires, fairs, weddings, eclipses, droughts, floods, harvests, new ages |
| `O` | Photograph mode (time stops, tilt-shift blur): `K` changes the light, `Enter` saves a PNG |
| `X` | Record a video with sound; press again to stop and save it |
| `G` / `V` | Next gallery world / the world of the day (seeded by today's UTC date) |
| `L` | Copy a link to this exact spot and moment |
| `Y` | Time-lapse (a day in five seconds, `TIMELAPSE_SPEED = 240`) |
| `I` | Turn looking inside houses (when zoomed right in) on / off |
| `H` | Hide the HUD |
| `?` or `/` | Help |
| `Esc` | Close overlays, leave photo mode, let go of a followed villager |

**Touch**: one finger pans, two fingers pan and pinch to zoom, a tap is a click.
Touch screens get a ☰ menu (`web/menu.js`) whose buttons press the same keys and
show what each option is currently set to; it appears for four seconds after a
touch.

### Recording video

`X` starts recording the canvas, and the soundscape once audio has started
(it is silent if you haven't clicked yet), at up to 60 fps and 12 Mbit/s with
`MediaRecorder`. Press `X` again and the browser downloads `meeple-<time>.mp4`
(`.webm` where MP4 isn't supported; Safari and recent Chrome write MP4, Firefox
WebM). The touch menu has a "Record video" button that shows on or off. Photo mode
and the HUD toggle keep working while recording, so you can hide the HUD first
(`H`). The video follows the canvas, so `pixel=` and quality tiers show up in it.

## URL options

All are optional query parameters (`main.rs`: `opts()`):

| Option | Meaning |
| --- | --- |
| `seed=1234` | The world. A number, or any text (hashed). Default: random |
| `quality=auto\|high\|medium\|low\|0..5` | `auto` (default) starts at Ultra and steps down if frames run slow (TV browsers start at tier 1); `high` = 0, `medium` = 2, `low` = 4, or a number pins that tier |
| `fxoff=ssao,dof,...` | Turn named effects off whatever the tier says: `cascades ssao rays dof taa grade sky water cab fog clouds materials bounce particles air` |
| `pixel=1..6` | 1 is crisp; 2 (default) to 6 render at lower resolution for chunky pixels |
| `day=1200` | Real seconds per in-game day (default 1200, minimum 30). A season is 3 days |
| `season=spring\|summer\|autumn\|fall\|winter` or a number | Start season (a number is the fractional 0..4 season) |
| `hour=18` | Start hour (default 7.5) |
| `x=…&z=…` | Where to start the camera (else beside the town nearest the origin) |
| `t=…` | The exact moment, in absolute in-game days (written by `L`) |
| `yaw=…&zoom=…` | The exact view (zoom clamped 8–130) (written by `L`) |
| `pitch=0.3` | Start looking this far down (0.15–1.2); the autopilot takes over from there |
| `aurora` or `aurora=0.5` | Northern lights at that strength regardless of weather and place |
| `prof` | Log where each frame's CPU time goes, every five seconds |
| `bench` | Time the terrain noise one point at a time against four at a time (and a whole chunk) in this browser |
| `bella` | An easter egg: a man called Bella lifts weights in front of a mirror in every town square |

## What the browser remembers

Each seed's clock is kept in `localStorage` (keys `meeple.<name>.<seed>`,
`app/persist.rs`), so coming back to a world resumes where it left off, plus up to
three days (`MAX_AWAY_DAYS`) for the time away, with a welcome line. This is skipped
if `hour`, `season` or `t` is given. Nothing else is stored; the server is static and
sees nothing.

## Splash screen

A pixel-art title card ("Meeple", a red meeple, and "by Joe" with a smiley) covers
the page from first paint, while the wasm downloads and the world is built.
`web/splash.js` draws it on a tiny 40×36 canvas from bitmap glyphs defined in the
file, and CSS scales it up with `image-rendering: pixelated`. The letters drop in
one by one and the meeple hops once.

It stays for at least 2.4 s, then fades out (0.9 s) once the app has drawn its
first frame. `main.rs` signals that by setting `window.__meepleReady` after the
first `frame()`. Clicking the splash skips the minimum hold once the app is ready;
after 20 s it leaves regardless, so a failed start never hides the page forever.
Its styles (`#splash`) are in `web/style.css`.

## HUD and overlays

- **HUD** (top): the town the camera is in or the lie of the land, time and
  date, weather, and what the autopilot is following or heading for.
- **Toasts**: short messages ("Picture saved", looking-inside toggles).
- **Villager card**: name, trade, home, kin and what they are doing.
- **Newsletter, chronicle, map**: toggled with `J`, `C`, `Z` and refreshed a few times
  a second.
- **Postcards**: captioned holds on a lovely view.
- **Quality badge**: bottom-right, names the new tier each time the renderer steps
  down.
- **Update banner**: if a new build is published while you are fullscreen or have
  sound on, you are asked to reload rather than surprised (see
  [Build and deploy](build-and-deploy.md)).

## Quality tiers

See [Rendering](rendering.md) for what each tier draws.
