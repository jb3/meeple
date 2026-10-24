// Meeple soundscape, Generative music, part 1: scales, the music's own randomness, harmony, the pulse, energy and the step sequencer.
// One of the `web/audio/*.js` scripts, loaded in order as classic scripts so they
// share one global scope (see docs/audio.md).

// ---------- generative music ----------
// One slow pulse (a tempo of the world's own) carries everything. Warm pads
// voice-lead through chords picked by a small Markov chain; a Rhodes-like
// lead sings chord-aware phrases that come back varied (inverted,
// transposed, answered); and a wandering "energy" decides how much else
// joins in: shaker, heartbeat, arpeggios, a pulsing bass, voices. Quiet
// sections are the old drifting ambience, busy ones are nearly a band.
// Dawn climbs in arpeggios and dusk hangs on suspended chords.
// Every musical choice draws from a generator seeded by the world, so each
// world keeps its own tempo, swing, favourite chords and signature theme.

const PENTA = [0, 2, 4, 7, 9];
let ROOT = 48; // C3, shifted to each place's own key
// each place hums its own little motif now and then, in its own key
const PLACE_KEYS = [0, 2, -3, 5, -5, 7, -2, 4];
const keyNow = () => 48 + PLACE_KEYS[Math.floor(p[P.PLACE] * PLACE_KEYS.length) % PLACE_KEYS.length];
const MOTIFS = [[0, 2, 1, -1], [0, 1, 3, 2], [0, -1, -2, 0], [0, 2, 4, 2], [0, 3, 1, 2], [0, -2, -1, 1]];
const MOTIF_RHY = [[1, 1, 2, 2], [2, 1, 1, 2], [1, 2, 1, 1], [1, 1, 1, 3]];
const MUSIC_LEVEL = 0.75;

const midi = (n) => 440 * Math.pow(2, (n - 69) / 12);
const clampN = (x, a, b) => Math.min(b, Math.max(a, x));
let melodyBus = null;
let musicDelay = null;
let melodyStep = 7; // index into the scale ladder below

function musicSetup() {
  melodyBus = ctx.createGain();
  melodyBus.gain.value = 1;
  melodyBus.connect(music);
  // feedback delay with a darkening filter, panned slightly apart; its time
  // is set to a dotted eighth of the world's tempo once that is known
  const delay = ctx.createDelay(2);
  delay.delayTime.value = 0.46;
  musicDelay = delay;
  const fb = ctx.createGain();
  fb.gain.value = 0.38;
  const tone = ctx.createBiquadFilter();
  tone.type = "lowpass";
  tone.frequency.value = 1800;
  const wet = ctx.createGain();
  wet.gain.value = 0.4;
  const pan = ctx.createStereoPanner();
  pan.pan.value = 0.35;
  melodyBus.connect(delay);
  delay.connect(tone).connect(fb).connect(delay);
  tone.connect(wet).connect(pan).connect(music);
}

// each kind of country plays its own scale: the pentatonic of fields and
// woods, a phrygian-dominant in the desert, bare open fourths up high
const SCALES = [PENTA, [0, 1, 4, 5, 7, 8, 10], [0, 2, 7, 9], PENTA];
const scaleNow = () => SCALES[Math.round(p[P.BIOME])] || PENTA;

function ladder(i) {
  const sc = scaleNow();
  const oct = Math.floor(i / sc.length);
  return sc[((i % sc.length) + sc.length) % sc.length] + oct * 12;
}

/// Desert: a plucked, nasal oud-like note.
function oud(freq, t, len) {
  tone(melodyBus, "sawtooth", freq, t, 0.004, len * 1.6, 0.05, { filter: "bandpass", ff: 1300, q: 1.2, pan: rnd(-0.3, 0.3) });
  tone(melodyBus, "triangle", freq * 2.003, t, 0.004, len, 0.025);
}

/// Forest: a breathy flute.
function flute(freq, t, len) {
  tone(melodyBus, "sine", freq, t, 0.08, len * 1.5, 0.07, { filter: "lowpass", ff: 2600, pan: rnd(-0.3, 0.3) });
  noiseHit(melodyBus, t, "bandpass", freq * 2, 4, 0.06, len * 1.2, 0.012);
}

// ---- the world's own randomness ----

let rngState = 1;
/// mulberry32: a small seeded generator, so a world always composes alike
const mr = () => {
  rngState = (rngState + 0x6d2b79f5) | 0;
  let z = Math.imul(rngState ^ (rngState >>> 15), 1 | rngState);
  z = (z + Math.imul(z ^ (z >>> 7), 61 | z)) ^ z;
  return ((z ^ (z >>> 14)) >>> 0) / 4294967296;
};
const mrange = (a, b) => a + mr() * (b - a);
const mpick = (arr) => arr[Math.floor(mr() * arr.length)];

// ---- harmony ----

// a chord on each scale degree (semitones above the key note), as intervals
const CHORD_Q = {
  0: [0, 4, 7, 11], 2: [0, 3, 7, 10], 4: [0, 3, 7, 10], 5: [0, 4, 7, 11],
  7: [0, 4, 7, 10], 9: [0, 3, 7, 10], 10: [0, 4, 7, 14],
};
// where each chord likes to go next (weights)
const TRANS = {
  0: { 5: 3, 9: 3, 7: 2, 2: 2, 4: 1, 10: 1 },
  2: { 7: 3, 5: 2, 9: 1, 0: 1 },
  4: { 9: 3, 5: 3, 2: 1 },
  5: { 0: 3, 7: 3, 2: 2, 9: 1, 10: 1 },
  7: { 0: 4, 9: 3, 5: 2, 4: 1 },
  9: { 5: 3, 2: 3, 7: 1, 4: 2, 0: 1 },
  10: { 5: 3, 0: 3, 7: 1 },
};
const FAVOURITES = [0, 2, 4, 5, 7, 9, 10];
const ARP_PATS = [[0, 1, 2, 3, 2, 1, 2, 1], [0, 2, 1, 3, 2, 1, 3, 2], [0, 1, 2, 1, 3, 2, 1, 0], [0, 2, 3, 2, 1, 2, 3, 1]];
const BASS_PATS = [[1, 0, 0, 1, 0, 0, 1, 0], [1, 0, 1, 0, 0, 1, 0, 1], [1, 0, 0, 0, 1, 0, 0, 1]];

const PERMS = (() => {
  const out = [];
  const rec = (a, rest) => {
    if (!rest.length) out.push(a);
    rest.forEach((x, i) => rec([...a, x], [...rest.slice(0, i), ...rest.slice(i + 1)]));
  };
  rec([], [0, 1, 2, 3]);
  return out;
})();

/// The voicing of a four-note chord (pitch classes `pcs`) that moves each
/// voice the least from `prev`, kept between `lo` and `hi`.
function lead(prev, pcs, lo, hi) {
  let best = null;
  let cost = Infinity;
  for (const perm of PERMS) {
    let c = 0;
    const notes = [];
    for (let v = 0; v < 4; v++) {
      let d = (((pcs[perm[v]] - prev[v]) % 12) + 12) % 12;
      if (d > 6) d -= 12;
      let n = prev[v] + d;
      while (n > hi) n -= 12;
      while (n < lo) n += 12;
      c += Math.abs(n - prev[v]);
      notes.push(n);
    }
    // two voices on one pitch would just thin the chord
    c += (4 - new Set(notes).size) * 30;
    if (c < cost) {
      cost = c;
      best = notes;
    }
  }
  return best.sort((a, b) => a - b);
}

let persona = null;
let curDeg = 0;
let voicing = [48, 52, 55, 59];
let chordNotes = voicing;
let chordCount = 0;
let nextChordStep = 0;
let nextPhraseStep = 8;
let dawnArp = 0;

function nextDeg(night) {
  const row = TRANS[curDeg] || TRANS[0];
  const w = [];
  let total = 0;
  for (const k in row) {
    const d = +k;
    let x = row[k];
    if (persona.favour.includes(d)) x *= 1.7;
    if (night && d === 9) x *= 2;
    if (!night && d === 0) x *= 1.4;
    w.push([d, x]);
    total += x;
  }
  let r = mr() * total;
  for (const [d, x] of w) {
    r -= x;
    if (r <= 0) return d;
  }
  return w[0][0];
}

// ---- the pulse ----

let g0 = 0; // when step 0 sounds
let stepI = 0;
const beatLen = () => 60 / persona.bpm;
const stepLen = () => beatLen() / 2;
/// When step `i` (an eighth note) sounds: odd ones lag a little, for swing
function stepAt(i) {
  const s = stepLen();
  return g0 + Math.floor(i / 2) * 2 * s + (i & 1 ? s * (1 + persona.swing) : 0);
}

// ---- energy: ambient drifting to nearly a band, and back ----

let energy = 0.2;
let eBase = 0.2;
let nextSection = 0;
let lastE = 0;
let rhythmGate = 1; // an event's own music takes the rhythm over

function updateEnergy(t) {
  const dt = clampN(t - lastE, 0, 2);
  lastE = t;
  if (t > nextSection) {
    const r = mr();
    eBase = r < 0.35 ? mrange(0.05, 0.28) : r < 0.75 ? mrange(0.3, 0.6) : mrange(0.62, 0.95);
    nextSection = t + mrange(40, 110);
  }
  const hr = p[P.HOUR];
  const calm = 1 - Math.max(p[P.RAIN] * 0.6, p[P.SNOW] * 0.5, p[P.STORM] * 0.9, p[P.FOG] * 0.4, p[P.NIGHT] * 0.45);
  const social = Math.min(1, Math.min(p[P.CROWD], 1) * 0.4 + p[P.TAVERN] * 0.3 + p[P.CHILDREN] * 0.2);
  const twilight = Math.max(0, 1 - Math.abs(hr - 6.6) / 1.5, 1 - Math.abs(hr - 18.6) / 1.5);
  const target = Math.min(1, eBase * calm + social * 0.35 + twilight * 0.15);
  energy += (target - energy) * (1 - Math.exp(-dt / 8));
}

/// The music follows the weather: a music box in snow, soft piano in
/// rain, low drones in a storm; otherwise the hour of the day decides.
function mood() {
  if (p[P.STORM] > 0.4) return "storm";
  if (p[P.SNOW] > 0.2) return "snow";
  if (p[P.RAIN] > 0.3) return "rain";
  const hr = p[P.HOUR];
  if (hr > 5 && hr < 8) return "dawn";
  if (hr > 17 && hr < 20.3) return "dusk";
  return p[P.NIGHT] > 0.5 ? "night" : "day";
}

function genTheme() {
  const len = 4 + Math.floor(mr() * 3);
  const out = [{ d: 0, n: mpick([1, 2]) }];
  for (let k = 1; k < len; k++) out.push({ d: mpick([-2, -1, -1, 1, 1, 2, 3]), n: mpick([1, 1, 2, 2, 3]) });
  out[len - 1].n = mpick([2, 3, 4]);
  return out;
}

function seedMusic(seed, t) {
  rngState = (Math.imul(seed | 0, 0x9e3779b1) ^ 0x85ebca6b) | 0;
  for (let k = 0; k < 4; k++) mr();
  persona = {
    seed,
    bpm: mrange(58, 76),
    swing: mrange(0, 0.14),
    favour: [mpick(FAVOURITES), mpick(FAVOURITES)],
    arp: Math.floor(mr() * ARP_PATS.length),
    bass: Math.floor(mr() * BASS_PATS.length),
    ghost: mpick([3, 5, 7]),
    theme: genTheme(),
  };
  musicDelay.delayTime.setValueAtTime(beatLen() * 0.75, t);
  g0 = t + 0.2;
  stepI = 0;
  nextChordStep = 0;
  nextPhraseStep = 10;
  ROOT = keyNow();
  voicing = [ROOT + 4, ROOT + 7, ROOT + 11, ROOT + 14];
  curDeg = p[P.NIGHT] > 0.5 ? 9 : 0;
  chordCount = 0;
  lastPhrase = null;
  answerNext = false;
  energy = 0.2;
  eBase = 0.2;
  nextSection = t + mrange(25, 50);
  lastE = t;
}

function musicTick(t) {
  const seed = Math.round(p[P.SEED]);
  if (!persona || persona.seed !== seed) seedMusic(seed, t);
  updateEnergy(t);
  // a stalled page leaves the clock behind: pick the beat up again
  if (stepAt(stepI) < t - 0.6) {
    stepI = Math.ceil((t - g0) / stepLen() / 8) * 8;
    nextChordStep = stepI;
    nextPhraseStep = stepI + 4;
  }
  while (stepAt(stepI) < t + 0.4) {
    musicStep(stepI, stepAt(stepI));
    stepI++;
  }
}

const DENSITY = { day: 0.75, dawn: 0.7, dusk: 0.65, night: 0.55, rain: 0.45, snow: 0.7, storm: 0.15 };

function musicStep(i, at) {
  const m = mood();
  const night = p[P.NIGHT] > 0.5 || m === "storm";
  if (i >= nextChordStep) newChord(i, at, m, night);
  const pos = i & 7;
  const e = Math.min(energy * rhythmGate, p[P.TIER] >= 3 ? 0.45 : 1);

  if (dawnArp > 0) {
    const k = 8 - dawnArp;
    pluck(midi(chordNotes[k % 4] + 12 * (1 + Math.floor(k / 4))), at, 1.1, 0.05, melodyBus);
    dawnArp--;
  }

  if (m !== "storm") {
    // a shaker on the eighths, the offbeats lifted
    if (e > 0.3) noiseHit(music, at, "highpass", 7500, 0.8, 0.002, i & 1 ? 0.04 : 0.025, (0.008 + 0.018 * e) * (i & 1 ? 1.4 : 0.8));
    // a heartbeat on one and three, a ghost note later in the bar
    if (e > 0.42 && (pos === 0 || (e > 0.65 && pos === 4) || (e > 0.8 && pos === persona.ghost))) {
      tone(music, "sine", 78, at, 0.004, 0.3, 0.09 + 0.06 * e, { to: 44 });
    }
    // a soft backbeat
    if (e > 0.55 && (pos === 2 || pos === 6)) noiseHit(music, at, "bandpass", 1900, 1.1, 0.001, 0.07, 0.02 + 0.02 * e);
    // arpeggios over the chord, every other step and then every step
    if (e > 0.35 && !(night && e < 0.5) && (e > 0.6 || (i & 1) === 0) && mr() > 0.12) {
      const k = ARP_PATS[persona.arp][pos];
      pluck(midi(chordNotes[k] + (e > 0.75 && pos >= 4 ? 24 : 12)), at, 0.4, 0.022 + 0.03 * e, melodyBus);
    }
    // the bass picks up a rhythm of its own
    if (e > 0.45 && BASS_PATS[persona.bass][pos]) {
      const f = midi(ROOT - 12 + curDeg + (pos >= 5 ? 7 : 0));
      tone(music, "sine", f, at, 0.006, 0.45, 0.1);
      tone(music, "triangle", f * 2, at, 0.004, 0.2, 0.03);
    }
    // every fourth chord is announced by a rising wash of air
    if (e > 0.7 && chordCount % 4 === 3 && pos === 0 && i + 8 === nextChordStep) {
      noiseHit(music, at, "bandpass", 400, 1.2, stepAt(i + 8) - at, 0.2, 0.04, 6000);
    }
  }

  if (i >= nextPhraseStep) {
    const cold = Math.round(p[P.BIOME]) === 2;
    const density = DENSITY[m] * (1 - p[P.RAIN] * 0.2) * (cold ? 0.7 : 1) * (1 + e * 0.5);
    let used = 0;
    if (mr() < density) used = phrase(i, night, m, e);
    const rest = (answerNext ? 2 : mrange(6, night ? 24 : 16) * (1 - 0.6 * e)) * (cold ? 1.8 : 1);
    nextPhraseStep = i + used + Math.max(2, Math.round(rest));
  }
}

function chordBars(m, night) {
  if (m === "dusk") return 4;
  return energy * rhythmGate > 0.55 ? 2 : night ? 4 : 3;
}

function newChord(i, at, m, night) {
  ROOT = keyNow();
  curDeg = nextDeg(night);
  // dusk hangs on suspended chords, wanting to resolve
  let ivs = CHORD_Q[curDeg];
  if (m === "dusk") ivs = ivs[1] === 3 || curDeg === 7 ? [0, 5, 7, 10] : [0, 2, 7, 11];
  const pcs = ivs.map((iv) => (((ROOT + curDeg + iv) % 12) + 12) % 12);
  voicing = lead(voicing, pcs, ROOT - 2, ROOT + 21);
  chordNotes = voicing;
  chordCount++;
  const bars = chordBars(m, night);
  nextChordStep = i + bars * 8;
  const secs = stepAt(nextChordStep) - at;
  const dur = secs + 3;
  const e = energy * rhythmGate;
  const rain = p[P.RAIN];
  const soft = m === "snow" ? 0.5 : m === "rain" ? 0.7 : 1;
  chordNotes.forEach((n) => pad(midi(n), at, dur, night || m === "dusk", rain, soft));
  if (e < 0.45) bass(midi(ROOT - 12 + curDeg), at, dur, night);
  if (m === "storm") drone(midi(ROOT - 12 + curDeg), at, dur);
  if (m === "dawn") dawnArp = 8;
  // a vowel hums along the chord when things are lively
  if (e > 0.6 && m !== "storm" && mr() < 0.6) {
    chordNotes.slice(-2).forEach((n) => choir(midi(n + 12), at, secs * 0.9, 0.3, music, 2));
  }
  tuneHum(curDeg, at);
}

/// Retune the wind's hum to the chord's root and fifth.
function tuneHum(deg, t) {
  loops.humA.f.frequency.setTargetAtTime(midi(ROOT + 12 + deg), t, 1.5);
  loops.humB.f.frequency.setTargetAtTime(midi(ROOT + 19 + deg), t, 1.5);
}
