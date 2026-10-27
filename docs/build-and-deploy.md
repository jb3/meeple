# Build, test and deploy

## Toolchain

- Rust (stable) with the `wasm32-unknown-unknown` target.
- [`trunk`](https://trunkrs.dev) to build and bundle the wasm and static assets.
- `wasm-opt` is run by Trunk on release builds (`data-wasm-opt="z"` with SIMD, bulk
  memory, nontrapping float-to-int, sign extension and mutable globals enabled in
  `index.html`).

`.cargo/config.toml` enables `+simd128` for the wasm target so the `wide` crate and
LLVM's auto-vectoriser use real 128-bit vector instructions (every current browser
has them). Elsewhere `wide` falls back to SSE, NEON or scalar code, so native tests
agree with the wasm build to rounding.

Dependencies (`Cargo.toml`): `wasm-bindgen`, `js-sys`, `web-sys` (with an explicit
feature list for the DOM/GL/Storage APIs used), `glam` (maths), `wide` (SIMD),
`console_error_panic_hook`. Release profile: `opt-level = 3`, `lto = true`,
`codegen-units = 1`. The dev profile is `opt-level = 2` because debug builds of the
generator are too slow to be usable.

## Makefile

```
make           build the release bundle into release/
make deploy    build, then upload to the web server
make serve     run the local dev server (trunk serve --release)
make test      run the native tests
make clean     remove build output
```

- `build` runs `trunk build --release` with its own `CARGO_TARGET_DIR`
  (`target/bundle`) and output directory (`release/`), so it does not fight a running
  `trunk serve` over `dist/` or over wasm-opt's scratch file (which once shipped an
  empty `.wasm`; the build also checks the `.wasm` is non-empty).
- It then writes `release/version.txt` containing two hashes: *everything* in the
  bundle, and *the shell* (everything except the wasm, its JS glue and the hashes
  `index.html` quotes for them). `web/update.js` polls this file.
- `deploy` rsyncs `release/` to `HOST:REMOTE_DIR` (defaults in the Makefile; override
  with `make deploy HOST=… REMOTE_DIR=…`), running rsync under `sudo` on the server
  and deleting files left from older builds. Adapt the defaults to your own server;
  any static host works.
- `PUBLIC_URL` defaults to `/`; set it if the bundle is served from a sub-path.

## index.html and the web shell

`index.html` is the Trunk entry point. It declares the page (canvas, HUD, help
table, overlays, menu), copies the static assets (`copy-file`/`copy-dir` links), links
the stylesheet, builds the Rust crate (`data-trunk rel="rust"`), and loads the scripts:

| File | Role |
| --- | --- |
| `web/audio/*.js` | The WebAudio engine (see [Audio](audio.md)); copied as the `audio/` directory |
| `web/menu.js` | Defines `window.meepleUi` (`snap()` saves the canvas as a PNG, `record()`/`recording()` record video) and the touch menu |
| `web/favicon.js` | Draws a favicon of a meeple dressed for the in-game season on a darker sky at night, hanging off the audio engine's per-frame hook (`meepleAudio.onFrame`) |
| `web/splash.js` | The pixel-art splash screen (see [Splash screen](usage.md#splash-screen)) |
| `web/wakelock.js` | Keeps the screen awake while the world is on show, re-acquiring the lock on return to the tab |
| `web/update.js` | Polls `version.txt` every minute; swaps in the new wasm in place if only the wasm changed, otherwise reloads (or prompts if the viewer is fullscreen or has sound on, then restores fullscreen after the reload) |
| `web/style.css` | All styling: HUD, help, card, overlays, menu, postcards, quality badge |
| `web/sounds.html` | The sound test page |
| `web/icon.svg`, `apple-touch-icon.png`, `og.png` | Icons and the social image |

### In-place updates

When `update.js` finds that only the wasm changed (the "shell" hash is the same), it
fetches the new `index.html`, imports the new glue module, fetches the wasm, bumps
`window.__meepleGen`, replaces the canvas and starts the new instance. The old
instance notices the generation counter has moved on (`main.rs`: `generation()`),
stops its animation loop and removes the listeners it added (`retire()`). The world
starts over, as it would after a reload.

## Tests

`make test` runs `cargo test --target <host>` (native, not wasm). Tests live next to
the code:

| Location | What they cover |
| --- | --- |
| `world/chunk/tests.rs`, `bridge_tests.rs`, `tree_tests.rs` | Chunk generation invariants (towns, bridges, ferries, ski runs premade and clear, trees, rails), wide water crossed by a ferry |
| `world/chunk/seed_hunt.rs` | `rank_seeds`, an `#[ignore]`d hunt for good starting seeds (the `GALLERY` in `app.rs` came from it), and `find_feature_spots`, which finds a place for each feature for [Seeds](seeds.md) |
| `noise4.rs` | SIMD noise agrees with the scalar versions; two `#[ignore]`d benchmarks |
| `sim/people/tests.rs` | Villagers: schedules, weddings, families, events staying on the streets, visits |
| `sim/people/visit.rs` | Promised festivals and weddings wait for the camera and then happen |
| `sim/traffic/tests.rs`, `train_tests.rs` | Trains, signals, junctions |
| `sim/fauna/tests.rs` | Animals |
| `sim/ferry.rs`, `sim/stations.rs`, `sim/wonders/places.rs` | Ferry shuttling, passengers boarding, quiet places drawing what belongs to the time |

Three tests are `#[ignore]`d (the seed hunt and two noise benchmarks). Run them
explicitly with `cargo test -- --ignored`.

## Conventions

- Code reads like its surroundings: terse comments that say *why*, named constants
  with doc comments near the top of a file.
- Files above roughly 600 lines are split into a parent and a directory of child
  modules (`foo.rs` + `foo/`). Children use `use super::*` and share the parent's
  imports and private items.
- Anything that must reproduce across sessions uses hashes of `(seed, position/day)`;
  anything cosmetic uses `Rng`.
- Draw functions take `&mut Boxes` and append; nothing keeps GPU state of its own.

## Repository notes

- `target/`, `dist/` and `release/` are build outputs and are git-ignored.
- `Cargo.lock` is committed (it is an application).
- The licence is MIT (`LICENSE`).
