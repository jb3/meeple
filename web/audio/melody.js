// Meeple soundscape, Generative music, part 2: melodies, phrases and the instruments that play them.
// One of the `web/audio/*.js` scripts, loaded in order as classic scripts so they
// share one global scope (see docs/audio.md).

// ---- melody ----

let lastPhrase = null;
let answerNext = false;
let answering = false;
const cloneNotes = (a) => a.map((n) => ({ ...n }));

function placeMotif() {
  const idx = (Math.floor(p[P.PLACE] * 37) + persona.seed) % MOTIFS.length;
  const rhy = MOTIF_RHY[idx % MOTIF_RHY.length];
  return MOTIFS[idx].map((o, k) => ({ d: k ? o - MOTIFS[idx][k - 1] : 0, n: rhy[k % rhy.length] }));
}

/// A new random walk, leaning up at dawn and down at dusk.
function wander(night, m) {
  const steps = m === "dawn" ? [-1, 1, 1, 1, 2, 2] : m === "dusk" ? [-2, -1, -1, -1, 1, 1] : [-2, -1, -1, 1, 1, 2];
  const out = [];
  const n = 3 + Math.floor(mr() * 4);
  for (let k = 0; k < n; k++) out.push({ d: mpick(steps), n: mpick(night ? [1, 2, 2] : [1, 1, 1, 2]) });
  return out;
}

/// A phrase again, but changed: the way a theme comes back.
function vary(ph, kind) {
  const out = cloneNotes(ph);
  const at = Math.floor(mr() * out.length);
  if (kind === 0) {
    out.forEach((n, k) => { if (k) n.d = -n.d; }); // upside down
  } else if (kind === 1) {
    out[0].d += mpick([-2, -1, 1, 2]); // shifted up or down
  } else if (kind === 2) {
    const x = mpick([-1, 1]); // one note moved, the rest kept
    out[at].d += x;
    if (out[at + 1]) out[at + 1].d -= x;
  } else if (kind === 3) {
    out[at].n = out[at].n === 1 ? 2 : 1; // a different rhythm
  } else if (kind === 4) {
    out.length = Math.max(3, out.length - 1); // cut short, with a new ending
    out.push({ d: mpick([-2, -1, 1, 2]), n: mpick([2, 3]) });
  } else {
    const offs = [];
    let acc = 0;
    out.forEach((n) => { acc += n.d; offs.push(acc); }); // backwards
    offs.reverse();
    const rhy = out.map((n) => n.n).reverse();
    return offs.map((o, k) => ({ d: k ? o - offs[k - 1] : 0, n: rhy[k] }));
  }
  return out;
}

function compose(night, m) {
  answering = false;
  if (answerNext && lastPhrase) {
    answering = true;
    return vary(lastPhrase, mpick([0, 1, 5]));
  }
  const r = mr();
  if (lastPhrase && r < 0.4) return vary(lastPhrase, Math.floor(mr() * 6));
  if (r < 0.58) return cloneNotes(persona.theme); // the world's own tune
  if (r < 0.74) return placeMotif();
  return wander(night, m);
}

/// The ladder step nearest `idx` that is a tone of the chord now sounding.
function nearChordTone(idx) {
  const pcs = chordNotes.map((n) => n % 12);
  for (const o of [0, 1, -1, 2, -2, 3, -3]) {
    const k = idx + o;
    if (k < 2 || k > 12) continue;
    if (pcs.includes((((ROOT + ladder(k)) % 12) + 12) % 12)) return k;
  }
  return idx;
}

/// Schedule a melodic phrase from step `i0`; returns how many steps it takes.
function phrase(i0, night, m, e) {
  const notes = compose(night, m);
  const base = ROOT + (night ? 12 : 24);
  const biome = Math.round(p[P.BIOME]);
  let step = i0;
  notes.forEach((nt, j) => {
    const last = j === notes.length - 1;
    melodyStep = clampN(melodyStep + nt.d, 2, 12);
    // the strong beats, the first note and the last all land on the chord
    if (j === 0 || last || (step & 3) === 0) melodyStep = nearChordTone(melodyStep);
    let n = nt.n;
    if (e > 0.5 && n > 1) n = Math.max(1, Math.round(n * 0.7));
    else if (night && e < 0.4) n = Math.ceil(n * 1.5);
    const t = stepAt(step);
    const secs = stepAt(step + n) - t;
    const f = midi(base + ladder(melodyStep));
    const decay = last ? 2.2 : Math.max(1.0, secs * 1.8);
    if (m === "snow") musicBox(f * 2, t);
    else if (m === "rain") piano(f, t, last ? 3.0 : 1.8);
    else if (biome === 1) oud(f, t, last ? 1.6 : 0.9);
    else if (biome === 3 && !night) flute(f, t, last ? 1.8 : 1.1);
    else if (biome === 2) musicBox(f, t);
    else keys(f, t, night, decay);
    // the lively sections double the tune an octave up, on pluck
    if (e > 0.7 && !night) pluck(f * 2, t + 0.02, 0.5, 0.025, melodyBus);
    step += n;
  });
  lastPhrase = notes;
  answerNext = !answering && mr() < 0.45;
  return step - i0;
}

/// Music box: a bright, quickly fading tine.
function musicBox(freq, t) {
  tone(melodyBus, "triangle", freq, t, 0.002, 0.7, 0.06, { pan: rnd(-0.3, 0.3) });
  tone(melodyBus, "sine", freq * 3.01, t, 0.002, 0.25, 0.015);
}

/// Soft piano: a few decaying harmonics.
function piano(freq, t, decay) {
  [1, 2, 3, 4.02].forEach((h, i) => {
    tone(melodyBus, "sine", freq * h, t, 0.006, decay / (1 + i * 0.8), 0.06 / (1 + i * 1.5));
  });
}

/// Low storm drone: two detuned saws under a dark filter.
function drone(freq, t, dur) {
  const f = ctx.createBiquadFilter();
  f.type = "lowpass";
  f.frequency.value = 260;
  const g = ctx.createGain();
  g.gain.setValueAtTime(0.0001, t);
  g.gain.linearRampToValueAtTime(0.06, t + dur * 0.4);
  g.gain.linearRampToValueAtTime(0.0001, t + dur);
  f.connect(g).connect(music);
  for (const det of [-9, 11]) {
    const o = ctx.createOscillator();
    o.type = "sawtooth";
    o.frequency.value = freq / 2;
    o.detune.value = det;
    o.connect(f);
    o.start(t);
    o.stop(t + dur + 0.1);
  }
}

function pad(freq, t, dur, night, rain, soft = 1) {
  const f = ctx.createBiquadFilter();
  f.type = "lowpass";
  f.Q.value = 0.6;
  const cut = (night ? 520 : 900) * (1 - rain * 0.3) * (1 + energy * 0.7);
  f.frequency.setValueAtTime(cut * 0.6, t);
  f.frequency.linearRampToValueAtTime(cut * 1.3, t + dur * 0.5);
  f.frequency.linearRampToValueAtTime(cut * 0.7, t + dur);
  const g = ctx.createGain();
  g.gain.setValueAtTime(0.0001, t);
  g.gain.linearRampToValueAtTime(0.026 * soft, t + dur * 0.35);
  g.gain.setValueAtTime(0.026 * soft, t + dur * 0.6);
  g.gain.linearRampToValueAtTime(0.0001, t + dur);
  f.connect(g).connect(music);
  for (const det of [-7, 6]) {
    const o = ctx.createOscillator();
    o.type = "sawtooth";
    o.frequency.value = freq;
    o.detune.value = det;
    o.connect(f);
    o.start(t);
    o.stop(t + dur + 0.1);
  }
}

function bass(freq, t, dur, night) {
  const o = ctx.createOscillator();
  o.type = "sine";
  o.frequency.value = freq;
  const o2 = ctx.createOscillator();
  o2.type = "triangle";
  o2.frequency.value = freq * 2;
  const g2 = ctx.createGain();
  g2.gain.value = 0.25;
  const g = ctx.createGain();
  g.gain.setValueAtTime(0.0001, t);
  g.gain.linearRampToValueAtTime(night ? 0.07 : 0.085, t + 1.8);
  g.gain.setValueAtTime(night ? 0.07 : 0.085, t + dur * 0.65);
  g.gain.linearRampToValueAtTime(0.0001, t + dur);
  o.connect(g);
  o2.connect(g2).connect(g);
  g.connect(music);
  for (const x of [o, o2]) {
    x.start(t);
    x.stop(t + dur + 0.1);
  }
}

/// Soft electric-piano note: FM sine whose brightness decays.
function keys(freq, t, night, decay) {
  const car = ctx.createOscillator();
  car.type = "sine";
  car.frequency.value = freq;
  const mod = ctx.createOscillator();
  mod.type = "sine";
  mod.frequency.value = freq;
  const idx = ctx.createGain();
  idx.gain.setValueAtTime(freq * (night ? 1.2 : 1.8), t);
  idx.gain.exponentialRampToValueAtTime(freq * 0.08, t + 0.6);
  mod.connect(idx).connect(car.frequency);
  const g = ctx.createGain();
  g.gain.setValueAtTime(0.0001, t);
  g.gain.exponentialRampToValueAtTime(0.075, t + 0.008);
  g.gain.exponentialRampToValueAtTime(0.0001, t + decay * (night ? 1.4 : 1));
  const pan = ctx.createStereoPanner();
  pan.pan.value = rnd(-0.3, 0.3);
  car.connect(g).connect(pan).connect(melodyBus);
  const end = t + decay * 1.4 + 0.1;
  for (const x of [car, mod]) {
    x.start(t);
    x.stop(end);
  }
}
