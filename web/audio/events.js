// Meeple soundscape, The music of events: fires, weddings, christenings and closing time each get a cue.
// One of the `web/audio/*.js` scripts, loaded in order as classic scripts so they
// share one global scope (see docs/audio.md).

// ---------- the music of events ----------
// Each event has its own band and its own tempo, but all of them play in
// the key of the music around them (ROOT), on its diatonic chords.

/// A soft plucked note: the harpsichord of the wedding march.
/// `bright` opens up (above 1) or darkens (below) the tone.
function pluck(freq, t, len, peak, dest, bright = 1) {
  tone(dest, "triangle", freq, t, 0.003, len, peak, { filter: "lowpass", ff: 3200 * bright });
  tone(dest, "sawtooth", freq * 2.002, t, 0.002, len * 0.4, peak * 0.25 * bright, { filter: "bandpass", ff: 2400 * bright, q: 1 });
}

/// Bowed strings: detuned saws that swell in slowly.
function strings(freq, t, len, peak, dest) {
  const f = ctx.createBiquadFilter();
  f.type = "lowpass";
  f.frequency.value = 1500;
  f.Q.value = 0.5;
  const g = ctx.createGain();
  g.gain.setValueAtTime(0.0001, t);
  g.gain.linearRampToValueAtTime(peak, t + 0.18);
  g.gain.setValueAtTime(peak, t + len * 0.7);
  g.gain.linearRampToValueAtTime(0.0001, t + len);
  f.connect(g).connect(dest);
  for (const det of [-9, 8]) {
    const o = ctx.createOscillator();
    o.type = "sawtooth";
    o.frequency.value = freq;
    o.detune.value = det;
    o.connect(f);
    o.start(t);
    o.stop(t + len + 0.1);
  }
}

// the diatonic triad on a scale degree (semitones above the key note)
const TRIAD_Q = { 0: [0, 4, 7], 2: [0, 3, 7], 4: [0, 3, 7], 5: [0, 4, 7], 7: [0, 4, 7], 9: [0, 3, 7], 11: [0, 3, 6] };
const triad = (deg) => TRIAD_Q[deg].map((iv) => deg + iv);

let chrVoicing = null;
let lastMel = 12;

/// A fire: the relative minor, a driving pedal, taiko, strings that climb
/// and a handbell sounding the alarm.
function cueFire(at, n) {
  const bar = Math.floor(n / 8);
  const pos = n % 8;
  const deg = [9, 5, 7, 4, 9, 5, 11, 4][bar % 8];
  const accent = pos === 0 ? 1 : pos === 3 || pos === 6 ? 0.7 : 0.4;
  tone(cueBus, "sawtooth", midi(ROOT - 3), at, 0.005, 0.17, 0.07 * accent, { filter: "lowpass", ff: 300 + 400 * accent });
  if (pos === 0 || pos === 3 || pos === 6) tone(cueBus, "sine", 85, at, 0.004, 0.3, 0.3, { to: 40 });
  if (pos === 2 || pos === 6) noiseHit(cueBus, at, "bandpass", 1100, 1, 0.002, 0.09, 0.07);
  if (pos & 1) noiseHit(cueBus, at, "highpass", 7000, 0.8, 0.002, 0.03, 0.03);
  if (pos === 0) {
    const tr = triad(deg);
    tr.forEach((iv) => strings(midi(ROOT + 12 + iv), at, 1.6, 0.03, cueBus));
    // the second time round the strings double an octave up
    if (n >= 64) tr.forEach((iv) => strings(midi(ROOT + 24 + iv), at, 1.6, 0.02, cueBus));
  }
  if (bar % 2 === 1 && pos < 3) {
    tone(cueBus, "triangle", midi(ROOT + 33), at, 0.002, 0.25, 0.06);
    tone(cueBus, "sine", midi(ROOT + 33) * 2.76, at, 0.002, 0.12, 0.02);
  }
  return 0.19;
}

/// A wedding: a harpsichord on a descending ground bass, a bowed tune
/// above it that quickens each time round, bells and a tambourine.
function cueWedding(at, n) {
  const bar = Math.floor(n / 8);
  const pos = n % 8;
  const cycle = Math.min(2, Math.floor(n / 64));
  const deg = [0, 7, 9, 4, 5, 0, 5, 7][bar % 8];
  const canon = [4, 2, 0, -1, -3, -5, -3, -1][bar % 8];
  const tr = triad(deg);
  // the harpsichord swells and fades over a long phrase and brightens and
  // mellows against it at a different rate, so no two rounds sound alike
  const round = n / 64;
  const swell = 0.75 + 0.45 * Math.sin(round * 2.1) * Math.sin(bar * 0.7 + 1);
  const bright = 1 + 0.2 * Math.sin(round * 1.3 + bar * 0.45);
  if (pos === 0) {
    pluck(midi(ROOT - 12 + deg), at, 1.2, 0.2, cueBus);
    tr.forEach((iv) => strings(midi(ROOT + iv), at, 2.2, 0.018, cueBus));
    if (bar % 8 === 0 && cycle > 0) bellStrike(at, midi(ROOT + 24), 0.5, undefined, cueBus);
  }
  if (pos === 4) pluck(midi(ROOT - 12 + deg + 7), at, 0.8, 0.13 * swell, cueBus, bright);
  const notes = [tr[0], tr[1], tr[2], tr[0] + 12];
  // the harpsichord figure changes shape every bar, rests here and there, and
  // once the tune is at full pace it thins out and keeps to the strong beats,
  // so it never settles into one up-and-down loop
  const shapes = [[0, 1, 2, 3, 2, 1, 2, 1], [0, 2, 1, 3, 1, 2, 0, 2], [3, 2, 1, 0, 1, 2, 3, 2], [0, 1, 2, 1, 3, 2, 1, 0], [2, 0, 1, 2, 3, 1, 2, 3]];
  const h = (Math.imul(bar + 1, 2654435761) >>> 0) / 4294967296;
  const shape = shapes[Math.floor(h * shapes.length)];
  const dense = cycle < 2 ? 0.8 : 0.45;
  const roll = ((Math.imul(n + 7, 1103515245) >>> 7) & 255) / 255;
  if (pos % 2 === 0 || roll < dense) {
    const hop = cycle === 2 && bar % 4 === 3 ? 12 : 0;
    // accents on the beat, and a ring that lengthens when it's soft
    const accent = pos % 4 === 0 ? 1.3 : 1;
    pluck(midi(ROOT + 12 + hop + notes[shape[pos]]), at, 0.4 + 0.2 / swell, 0.16 * swell * accent, cueBus, bright);
  }
  // the tune: one long, soft bowed note a bar following the descending line,
  // leaving the harpsichord to supply the movement
  if (pos === 0) {
    if (n < 64) {
      // the opening: the tune, bright and high
      fiddle(midi(ROOT + 24 + canon), at, 0.25 * 8 * 0.95, cueBus, true);
    } else {
      // then it settles an octave lower and duller, sitting on the chord's
      // own tones (the one nearest the line), holding them for the bar
      let best = tr[0];
      for (const x of [...tr, ...tr.map((v) => v - 12)]) {
        if (Math.abs(x - canon) < Math.abs(best - canon)) best = x;
      }
      fiddle(midi(ROOT + 12 + best), at, 0.25 * 8 * 0.95, cueBus, 650);
    }
  }
  if (cycle > 0 && (pos === 2 || pos === 6)) noiseHit(cueBus, at, "highpass", 7000, 0.8, 0.002, 0.05, 0.035);
  if (cycle > 1 && pos === 0) tone(cueBus, "sine", 80, at, 0.005, 0.2, 0.12, { to: 50 });
  return 0.25;
}

/// The wedding's last chord: the home chord held, a low bass note and a bell,
/// out on the effects bus so it rings on after the cue's own bus has gone.
function weddingFinale(at) {
  const tr = triad(0);
  pluck(midi(ROOT - 12), at, 2.5, 0.22, sfx);
  tr.forEach((iv) => {
    strings(midi(ROOT + iv), at, 3.6, 0.03, sfx);
    pluck(midi(ROOT + 12 + iv), at + 0.04, 1.6, 0.1, sfx, 0.8);
  });
  bellStrike(at + 0.1, midi(ROOT + 24), 0.7, undefined, sfx);
  duck(0.3, 1.2, 3.0, at);
}

/// A christening: a full choir in four parts over a slow I-vi-IV-V round, a
/// soprano descant, a harp, and a bell that peals at the end of each round.
function cueChristening(at, n) {
  const bar = Math.floor(n / 4);
  const pos = n % 4;
  const deg = [0, 9, 5, 7, 0, 5, 7, 0][bar % 8];
  const tr = triad(deg);
  if (pos === 0) {
    const pcs = [...tr, tr[0]].map((iv) => (((ROOT + iv) % 12) + 12) % 12);
    chrVoicing = lead(chrVoicing || [ROOT, ROOT + 4, ROOT + 7, ROOT + 12], pcs, ROOT - 5, ROOT + 16);
    const vowel = [0, 1, 0, 2][bar % 4];
    const swell = 1.3 + (bar % 8 >= 4 ? 0.5 : 0);
    // the parts come in one after another: bass, tenor, alto, soprano
    choir(midi(ROOT - 12 + deg), at, 4.4, swell * 0.8, cueBus, 1, 0.4);
    chrVoicing.forEach((note, k) => choir(midi(note), at + k * 0.12, 4.4, swell, cueBus, vowel));
    choir(midi(chrVoicing[3] + 12), at + 0.4, 4.0, swell * 0.55, cueBus, 0, 1);
    // a small bell on the root, a harp running up the chord
    bellStrike(at + 0.05, midi(ROOT + 12 + deg), 0.3, undefined, cueBus);
    [tr[0], tr[1], tr[2], tr[0] + 12, tr[1] + 12].forEach((iv, k) => pluck(midi(ROOT + 12 + iv), at + k * 0.11, 1.4, 0.07, cueBus));
  }
  // a descant sung over the second half of every round
  if (bar % 8 >= 4 && (pos === 0 || pos === 2)) {
    const pick = pos === 0 ? tr[1] : tr[2];
    choir(midi(ROOT + 24 + (pick % 12)), at, pos === 0 ? 2.6 : 3.2, 0.8, cueBus, 0, 1.2);
  }
  // the peal: bells tumbling down the pentatonic scale
  if (bar % 8 === 7 && pos === 0) {
    [9, 7, 4, 2, 0].forEach((x, k) => bellStrike(at + 0.2 + k * 0.5, midi(ROOT + 12 + x), 0.35, undefined, cueBus));
  }
  return 0.9;
}

/// Closing time: a tipsy accordion waltz with a stomp, clinking glasses,
/// the odd slide of a note, and everyone singing along.
function cueClosing(at, n) {
  const bar = Math.floor(n / 3);
  const pos = n % 3;
  const deg = [0, 0, 5, 5, 7, 7, 0, 7][bar % 8];
  const tr = triad(deg);
  const drift = rnd(-45, 45);
  if (pos === 0) {
    tone(cueBus, "triangle", midi(ROOT - 12 + deg), at, 0.01, 0.35, 0.16, { detune: drift });
    tone(cueBus, "sine", 100, at, 0.004, 0.18, 0.14, { to: 55 });
  } else {
    tr.forEach((iv) => tone(cueBus, "square", midi(ROOT + iv), at, 0.03, 0.3, 0.035, { detune: drift + rnd(-14, 14), filter: "bandpass", ff: 1100, q: 0.9 }));
    noiseHit(cueBus, at, "bandpass", 1500, 1, 0.002, 0.05, 0.025);
  }
  // the tune, on the accordion's reedy pair: it wanders about the chord
  if (mr() < 0.85) {
    const cands = [tr[0] + 12, tr[1] + 12, tr[2] + 12, tr[0] + 24];
    let best = cands[0];
    for (const c of cands) if (Math.abs(c - lastMel) + mr() * 3 < Math.abs(best - lastMel) + mr() * 3) best = c;
    lastMel = best;
    const f = midi(ROOT + 12 + best);
    const slide = mr() < 0.07;
    tone(cueBus, "square", f, at, 0.02, 0.36, 0.04, { detune: drift + 9, filter: "bandpass", ff: 1500, q: 0.8, to: slide ? f * 0.84 : undefined });
    tone(cueBus, "sawtooth", f, at, 0.02, 0.36, 0.025, { detune: drift - 9, filter: "bandpass", ff: 1500, q: 0.8 });
  }
  if (pos > 0 && mr() < 0.14) {
    tone(cueBus, "sine", rnd(2600, 3600), at, 0.001, 0.25, 0.03);
    tone(cueBus, "sine", rnd(4800, 6000), at, 0.001, 0.12, 0.012);
  }
  // from the third bar on the whole room sings along
  if (n >= 9 && bar % 3 === 0 && pos === 0) {
    tr.forEach((iv) => choir(midi(ROOT + iv), at, 3.6, 0.45, cueBus, 1, 0.9));
  }
  return 0.43;
}

/// Music for what's going on nearby. It plays in the music's key and steps
/// the generative music back while it lasts.
function cueTick(t, kind) {
  if (kind !== cueKind) {
    cueKind = kind;
    cueStep = 0;
    next.cue = 0;
    chrVoicing = null;
    lastMel = 12;
  }
  if (next.cue < t) next.cue = t + 0.05;
  while (next.cue < t + 0.3) {
    const at = next.cue;
    const play = [null, cueFire, cueWedding, cueChristening, cueClosing][kind];
    next.cue += play ? play(at, cueStep) : 0.4;
    cueStep++;
  }
}
