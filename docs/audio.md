# Audio

Code: `web/audio/*.js` (the engine) and `src/app/frame/audio.rs` (the Rust side).

There are **no samples**. Everything is synthesised with WebAudio: oscillators,
filtered noise, formant "voices", convolution reverb, and a generative music
engine.

## Protocol

Once per frame Rust calls `meepleAudio.update(params, events)` (declared in
`src/app.rs` as `audio_update`):

- `params`: a `Float32Array` of 30 floats. The indices are the `P` table in
  `web/audio/core.js`.
- `events`: a string of `name:volume:pan:pitch;` records for one-shot sounds that
  happened this frame (`thunder:v;` and `chime:v:pan:code;` and `boom:v:pan:1;` use
  their own shapes). The pitch field carries the speaker's trade in the hundreds
  (`role = floor(pitch / 100)`), so voices can differ by trade.

`drive_audio` (`app/frame/audio.rs`) turns the frame's `Sound`s into events:
volume is attenuated by distance from the camera (`att`, scaled by zoom), bells and
horns carry further (`sqrt`), semaphore signals carry furthest (the signals stand out
from the crossings), and panned left-right by where the source is across the screen.
Sounds below volume 0.02 are dropped. Also computed per frame:

| Param | Meaning |
| --- | --- |
| `HOUR`, `NIGHT`, `SEASON` | Time of day, darkness, season (0..4) |
| `RAIN`, `SNOW`, `WIND`, `STORM`, `FOG` | Weather (rain and snow are split by whether it is snowing) |
| `CROWD` | Visible villagers near the camera ÷ 14 |
| `WATER`, `NATURE` | Water and vegetation amounts in nearby chunks (from `Features`) |
| `TRAIN`, `TRAIN_SPEED` | Nearest train's loudness and speed |
| `FESTIVAL`, `CHOIR` | Nearest gathering's band and choir loudness |
| `PLACE` | A hash of the nearest town's id, 0..1: each town has its own key and tune |
| `ZOOM` | Camera zoom (close in = small) |
| `MILL`, `SAIL`, `MILL_PAN` | Nearest windmill, its sail angle, and its stereo position |
| `WHEEL` | Nearest waterwheel |
| `BIOME` | 0 plains, 1 desert, 2 cold or high, 3 forest or jungle (picks the musical scale) |
| `CUE`, `CUE_LEVEL` | An event nearby with its own music (1 fire, 2 wedding, 3 christening, 4 closing time) and how near |
| `TAVERN`, `CHILDREN`, `ENCLOSED`, `CHURCH` | The inn, children at play, how enclosed the streets are, the church close by |
| `TIER` | The renderer's quality tier, so the mix can skip what it cannot afford |
| `SEED` | The world's seed (each world keeps its own musical character) |

The church clock also strikes the hours and one soft stroke on the half (the
chime code carries the hour); the strike's loudness and pan follow the nearest
church.

## File layout

The engine is eight classic scripts loaded in order (see `index.html` and
`web/sounds.html`). They share **one global scope** (`let`/`const`/`function` at the
top level of each file are visible to the next), so load order matters and the names
are deliberately unique. They replace what used to be a single 2000-line IIFE.

| File | Contents |
| --- | --- |
| `core.js` | `P`, shared state (`ctx`, buses, `loops`), `start()` which builds the audio graph (including a `MediaStreamAudioDestinationNode` tapped after the compressor, for video recording; master, sfx/amb/music/voice/folk buses, reverb sends, rooms, ducking), noise and impulse-response generators, the `set` helper for smooth parameter changes |
| `voices.js` | Envelope and tone helpers (`env`, `tone`, `syllable`, `panner`, `noiseHit`), bells, keyed pitches, speech timbres per trade, and the `voices` table of named one-shot sounds |
| `ambience.js` | `scheduled()`: sounds that happen on their own timers (birds, crickets, frogs, woodpecker, owls, drips, wind gusts, train, mill and wheel, children, tavern, enclosed streets, church) plus `chirp` and `cricket` |
| `music.js` | The generative music engine: scales, the music's own seeded randomness, harmony (chord transitions), the pulse (step sequencer), energy, `musicTick`/`musicStep` |
| `melody.js` | Melody: motifs, phrase composition and variation (`compose`, `phrase`, `vary`, `wander`) and the instruments (`musicBox`, `piano`, `drone`, `pad`, `bass`, `keys`) |
| `folk.js` | The evening gathering: the jig (`jigTick`), `fiddle`, `choir` (formant-synthesised vowels) |
| `events.js` | The music of events: `cueTick` and the fire, wedding (with finale), christening and closing-time cues, `pluck`, `strings` |
| `update.js` | `update(params, events)`: smooth the parameters into the mix each frame, parse the event string, call `scheduled()`; user controls (`M`, `B`, `F`, resume on first gesture); and `window.meepleAudio = { update, P, voices, onFrame, state, captureStream }` |

`captureStream()` returns that tap's `MediaStream` (null before audio starts) for `meepleUi.record()`.

`onFrame` is a hook that `favicon.js` sets to receive each frame's params. `state()`
reports `{ muted, music }` for the touch menu.

## Sound design

**Weather and nature.** Layered noise loops for rain (heavier in a downpour), snow (a
muffled hush), wind (gusting), thunder (delayed claps, rolling), running water
(by the water amount), and drips. Birdsong by day (rate falls in rain and winter; a
dawn chorus swells at sunrise), crickets on warm nights (summer most; none in
winter), frogs near water on spring/summer nights, a woodpecker in woods, owls, a
wolf howl, gulls.

**Villagers.** Chatter is synthesised from formant "syllables" with a pitch that
depends on the speaker (`Person::voice`, and the trade carried in the event's pitch field), several styles (`chat`,
`chat_heart`, `chat_laugh`, `chat_note`), giggles from children, cheers, the town
crier. Doors, anvil clangs, hisses of steam and splashes are modelled too.

**Animals.** `bark`, `meow`, `cluck`, `baa`, `moo`, `quack`, `hoot`, `howl`,
`gull`, `sheepbell`, `woodpecker`, `clop` (hoofbeats).

**Machines.** Steam train: accented chuffs, rail-joint clickety-clack, a simmer and
pump at rest, horn and whistle, with a doppler shift as it nears or leaves;
semaphore clunks, bells and the church chime, windmill creak, waterwheel, foghorn,
balloon burner, calliope, fireworks `boom`, `ghost` (cold lights wander), `drone`
(low pad).

**Mix.** Distance-muffled sound (`air` fx: distant sounds lose their treble, more
so as the camera zooms out), reverb whose level follows how enclosed the streets are
and how close a church is, and music ducking under busy scenes and loud events. The
renderer's tier is passed in so tiers 3 and above skip the costlier voices and keep
the music energy lower.

## Music

The music engine is generative and seeded:

- **A persona** per world (`seedMusic(seed)`): tempo (58–76 bpm), swing, favourite
  chords, an arpeggio and bass pattern, a "ghost" note, and a short **theme**
  (4–6 notes) from `genTheme`. The world's seed determines all of it with a small
  private RNG (`mr`), so a world always sounds like itself.
- **A key per place**: `keyNow()` picks the root from the town's `PLACE`, and
  **scale** from the biome (`SCALES`): a major pentatonic for plains and forest, a
  seven-note scale with a flat second for deserts, and a sparse four-note scale for
  cold or high country.
- **Harmony**: chord degrees move by weighted transitions (`TRANS`); night favours
  the sixth, day the tonic.
- **Energy** (0.2 to near a band and back) drifts in sections (`updateEnergy`);
  rhythm layers (arpeggio, bass, keys, drone, pad) fade in with it. An event's own
  music takes the rhythm over (`rhythmGate`).
- **Melody**: phrases composed from the theme with variation, with call and
  response (`answering`), voiced against the current chord.
- **Evening gathering**: a jig on the fiddle and a choir whose loudness follows the
  nearest gathering.
- **Event cues** (`events.js`): `cueFire` (a driving bass and drum pulse under strings), `cueWedding` (a harpsichord figure over
  a long bowed tune, with a `weddingFinale`), `cueChristening` (a descant and a peal
  of bells tumbling down the pentatonic scale) and `cueClosing` (a tune on the
  accordion's reedy pair that the whole room sings along to from the third bar).

The music's overall level drops with how busy the scene is
(`MUSIC_LEVEL × (1 − busy × …)`). `B` turns music off; `M` mutes everything. Audio
starts on the first user gesture (browsers refuse earlier).

## The sound test page

`web/sounds.html` loads the real engine and lists every sound it can make as a
button (voices, ambient sounds, and event cues) so any of them can be auditioned
without waiting for it to happen. It is copied into the bundle as `/sounds.html`.
