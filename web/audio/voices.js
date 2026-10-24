// Meeple soundscape, One-shot voices: the named sounds Rust asks for in the `events` string, plus the shared envelope, tone and noise helpers.
// One of the `web/audio/*.js` scripts, loaded in order as classic scripts so they
// share one global scope (see docs/audio.md).

// ---------- one-shot voices ----------

function env(g, t, a, peak, d) {
  g.gain.setValueAtTime(0.0001, t);
  g.gain.exponentialRampToValueAtTime(Math.max(peak, 0.0002), t + a);
  g.gain.exponentialRampToValueAtTime(0.0001, t + a + d);
}

function tone(dest, type, freq, t, a, d, peak, opts = {}) {
  const o = ctx.createOscillator();
  o.type = type;
  o.frequency.setValueAtTime(freq, t);
  if (opts.to) o.frequency.exponentialRampToValueAtTime(opts.to, t + (opts.glide || a + d));
  if (opts.detune) o.detune.value = opts.detune;
  const g = ctx.createGain();
  env(g, t, a, peak, d);
  let node = o;
  if (opts.filter) {
    const f = ctx.createBiquadFilter();
    f.type = opts.filter;
    f.frequency.value = opts.ff || 1200;
    f.Q.value = opts.q || 1;
    node.connect(f);
    node = f;
  }
  node.connect(g).connect(opts.pan !== undefined ? panner(dest, opts.pan) : dest);
  o.start(t);
  o.stop(t + a + d + 0.05);
  return o;
}

/// One babble syllable whose pitch moves through `bend` over `len` seconds.
function syllable(t, type, bend, len, peak, formant, pn) {
  formant *= tm().fm;
  const o = ctx.createOscillator();
  o.type = type;
  o.frequency.setValueAtTime(bend[0], t);
  bend.slice(1).forEach((f, i) => o.frequency.linearRampToValueAtTime(f, t + (len * (i + 1)) / (bend.length - 1)));
  const f = ctx.createBiquadFilter();
  f.type = "bandpass";
  f.frequency.value = formant;
  f.Q.value = 1.5;
  const g = ctx.createGain();
  env(g, t, 0.01, peak, len);
  o.connect(f).connect(g).connect(panner(voice, pn));
  o.start(t);
  o.stop(t + len + 0.05);
}

function panner(dest, pan) {
  const pn = ctx.createStereoPanner();
  pn.pan.value = pan;
  // air soaks up treble over distance: the quieter an event arrives (the
  // further off it is), the duller it sounds
  if (cur.vol !== null && p[P.TIER] <= 2) {
    const lp = ctx.createBiquadFilter();
    lp.type = "lowpass";
    lp.frequency.value = 1400 + 17000 * Math.pow(Math.min(cur.vol, 1), 1.4);
    lp.Q.value = 0.5;
    pn.connect(lp).connect(dest);
  } else {
    pn.connect(dest);
  }
  return pn;
}

function noiseHit(dest, t, type, freq, q, a, d, peak, sweepTo, pn) {
  const src = ctx.createBufferSource();
  src.buffer = noiseBuf;
  const f = ctx.createBiquadFilter();
  f.type = type;
  f.frequency.setValueAtTime(freq, t);
  if (sweepTo) f.frequency.exponentialRampToValueAtTime(sweepTo, t + a + d);
  f.Q.value = q;
  const g = ctx.createGain();
  env(g, t, a, peak, d);
  src.connect(f).connect(g).connect(pn === undefined ? dest : panner(dest, pn));
  src.start(t, Math.random() * 3);
  src.stop(t + a + d + 0.05);
}

const rnd = (a, b) => a + Math.random() * (b - a);

/// One stroke of a bell: a struck metal ring with its hum tones.
function bellStrike(t, base, v, pn, dest = sfx) {
  [1, 2, 2.4, 3, 4.2, 5.4].forEach((m, i) => {
    tone(dest, "sine", base * m, t, 0.005, 4.5 / (1 + i * 0.6), (0.14 / (1 + i)) * v, pn === undefined ? {} : { pan: pn });
  });
}
/// The pentatonic note of the music's key nearest to `freq`
function keyed(freq) {
  const n = 69 + 12 * Math.log2(freq / 440);
  let best = Math.round(n);
  let bd = 99;
  for (let k = -6; k <= 6; k++) {
    const c = Math.round(n) + k;
    if (PENTA.includes((((c - ROOT) % 12) + 12) % 12) && Math.abs(c - n) < bd) {
      bd = Math.abs(c - n);
      best = c;
    }
  }
  return midi(best);
}
// the sound being played right now: where it is across the screen (-1..1)
// and, for villagers, their voice pitch
const cur = { pan: null, pitch: 1, role: 0, vol: null };
const pan = () => (cur.pan === null ? rnd(-0.7, 0.7) : cur.pan);
// each trade has its own sort of voice: lower or higher, rougher or softer,
// slower or quicker (cur.role, 1..12, rides along with the pitch)
const TIMBRES = {
  1: { mul: 0.9, fm: 0.85, type: "sawtooth", len: 1.1 },
  2: { mul: 1.0, fm: 1.0, len: 1 },
  3: { mul: 0.8, fm: 0.8, type: "sawtooth", len: 1 },
  4: { mul: 0.78, fm: 0.75, type: "sawtooth", len: 0.95 },
  5: { mul: 1.05, fm: 1.25, len: 0.9 },
  6: { mul: 0.9, fm: 0.9, type: "triangle", len: 1.5 },
  7: { mul: 1.0, fm: 1.15, len: 1 },
  8: { mul: 1.05, fm: 1.0, type: "sine", len: 1.1 },
  9: { mul: 1.1, fm: 1.1, len: 0.8 },
  10: { mul: 1, fm: 1, len: 1 },
  11: { mul: 1, fm: 1.3, len: 0.85 },
  12: { mul: 0.95, fm: 0.85, len: 1.25 },
};
const tm = () => TIMBRES[cur.role] || TIMBRES[10];
const voicePitch = () => 235 * cur.pitch * tm().mul * rnd(0.97, 1.03);

const voices = {
  thunder(v) {
    // sound lags the flash: the quieter (the further off), the later it comes
    const t = ctx.currentTime + 0.05 + (1.5 - Math.min(v, 1)) * rnd(1.2, 3.2);
    duck(0.45 * Math.min(v, 1), 0.4, 2.8, t);
    noiseHit(sfx, t, "lowpass", 900, 0.7, 0.02, 0.6, 0.85 * v, 200);
    noiseHit(sfx, t + 0.1, "lowpass", 260, 0.5, 0.3, rnd(3, 5), 1.3 * v, 60);
  },
  horn(v) {
    // two sustained blasts of a three-note chime: a long one, then a shorter one
    const t = ctx.currentTime;
    const pn = pan();
    duck(0.35 * Math.min(v, 1), 2.4, 1.6);
    for (const [start, hold] of [[0, 1.8], [2.15, 0.7]]) {
      const g = ctx.createGain();
      const peak = HORN_LEVEL * v;
      g.gain.setValueAtTime(0.0001, t + start);
      g.gain.linearRampToValueAtTime(peak, t + start + 0.08);
      g.gain.setValueAtTime(peak, t + start + hold);
      g.gain.linearRampToValueAtTime(0.0001, t + start + hold + 0.3);
      const f = ctx.createBiquadFilter();
      f.type = "lowpass";
      f.frequency.value = 1500;
      f.connect(g).connect(panner(sfx, pn));
      for (const freq of [311, 370, 466]) {
        const o = ctx.createOscillator();
        o.type = "sawtooth";
        o.frequency.value = freq;
        o.connect(f);
        o.start(t + start);
        o.stop(t + start + hold + 0.35);
      }
    }
  },
  bell(v) {
    bellStrike(ctx.currentTime, keyed(196 * cur.pitch), v, undefined);
    duck(0.3 * Math.min(v, 1), 0.3, 1.8);
  },
  // the church clock: the hour (cur.pitch is the count, 1 to 12), or one
  // soft stroke for the half hour (13)
  chime(v) {
    const t = ctx.currentTime;
    const pn = pan();
    const half = cur.pitch > 12.5;
    const n = half ? 1 : Math.max(1, Math.round(cur.pitch));
    duck(0.3 * Math.min(v, 1), Math.min(6, n * 1.1), 2.5);
    for (let i = 0; i < n; i++) bellStrike(t + 0.1 + i * 2.2, keyed(164), v * (half ? 0.5 : 0.85), pn);
  },
  // children at play: a few high, bright whoops
  giggle(v) {
    const t = ctx.currentTime;
    const pn = pan();
    const base = rnd(850, 1250);
    const n = 3 + Math.floor(Math.random() * 4);
    for (let i = 0; i < n; i++) {
      syllable(t + i * 0.09, "triangle", [base * (1 + i * 0.03), base * 0.9], 0.07, 0.07 * v, 2200, pn);
    }
  },
  gull(v) {
    const t = ctx.currentTime;
    const pn = pan();
    for (let i = 0; i < 2 + Math.floor(Math.random() * 2); i++) {
      tone(amb, "sawtooth", rnd(1500, 1900), t + i * 0.32, 0.02, 0.26, 0.05 * v, {
        to: rnd(800, 1000), filter: "bandpass", ff: 1500, q: 2, pan: pn,
      });
    }
  },
  // the fair's steam organ: bright, slightly reedy chords
  calliope(v) {
    const t = ctx.currentTime;
    const pn = pan();
    const base = 392 * cur.pitch;
    [1, 1.5, 2].forEach((m, i) => {
      tone(sfx, "sawtooth", base * m, t, 0.01, 0.3, (0.07 / (1 + i * 0.5)) * v, { pan: pn, filter: "lowpass", ff: 2600 });
    });
  },
  fiddle(v) {
    const t = ctx.currentTime;
    const pn = pan();
    const f = 440 * cur.pitch;
    tone(sfx, "sawtooth", f, t, 0.04, 0.22, 0.032 * v, { pan: pn, filter: "lowpass", ff: 2000 });
    tone(sfx, "sawtooth", f * 1.003, t, 0.04, 0.22, 0.02 * v, { pan: pn, filter: "lowpass", ff: 2000 });
  },
  cheer(v) {
    const t = ctx.currentTime;
    const pn = pan();
    for (let i = 0; i < 5; i++) {
      syllable(t + i * rnd(0.03, 0.09), "sawtooth", [rnd(220, 300), rnd(300, 420)], 0.45, 0.025 * v, rnd(700, 1400), pn);
    }
  },
  // an engine overhead: airship or biplane
  drone(v) {
    const t = ctx.currentTime;
    const pn = pan();
    const f = 70 * cur.pitch * (cur.pitch > 0.8 ? 1.8 : 1.4);
    tone(sfx, "sawtooth", f * 1.0, t, 0.15, 0.5, 0.14 * v, { pan: pn, filter: "lowpass", ff: 420 });
    tone(sfx, "sawtooth", f * 1.02, t, 0.15, 0.5, 0.12 * v, { pan: pn, filter: "lowpass", ff: 420 });
  },
  // the lighthouse's foghorn: a long, low, mournful blast
  foghorn(v) {
    const t = ctx.currentTime;
    const pn = pan();
    const g = ctx.createGain();
    g.gain.setValueAtTime(0.0001, t);
    g.gain.linearRampToValueAtTime(0.09 * v, t + 0.25);
    g.gain.setValueAtTime(0.09 * v, t + 1.9);
    g.gain.linearRampToValueAtTime(0.0001, t + 2.6);
    const f = ctx.createBiquadFilter();
    f.type = "lowpass";
    f.frequency.value = 340;
    f.connect(g).connect(panner(sfx, pn));
    for (const freq of [58, 87.5]) {
      const o = ctx.createOscillator();
      o.type = "sawtooth";
      o.frequency.setValueAtTime(freq * 1.06, t);
      o.frequency.linearRampToValueAtTime(freq, t + 0.4);
      o.connect(f);
      o.start(t);
      o.stop(t + 2.7);
    }
  },
  // a balloon's burner: a throaty gas roar that swells and dies away
  burner(v) {
    const t = ctx.currentTime;
    const pn = pan();
    noiseHit(sfx, t, "bandpass", 700, 0.9, 0.12, 1.1, 0.3 * v, 1100, pn);
    noiseHit(sfx, t, "lowpass", 260, 0.7, 0.1, 1.0, 0.25 * v, undefined, pn);
  },
  // a haunted ruin's moan: a thin wavering voice sliding down, with a shiver
  ghost(v) {
    const t = ctx.currentTime;
    const pn = pan();
    const g = ctx.createGain();
    g.gain.setValueAtTime(0.0001, t);
    g.gain.linearRampToValueAtTime(0.34 * v, t + 0.9);
    g.gain.linearRampToValueAtTime(0.0001, t + 3.2);
    const f = ctx.createBiquadFilter();
    f.type = "bandpass";
    f.frequency.value = 520;
    f.Q.value = 2.5;
    f.connect(g).connect(panner(sfx, pn));
    const lfo = ctx.createOscillator();
    const depth = ctx.createGain();
    lfo.frequency.value = 5.5;
    depth.gain.value = 9;
    lfo.connect(depth);
    lfo.start(t);
    lfo.stop(t + 3.3);
    for (const freq of [330, 332.5]) {
      const o = ctx.createOscillator();
      o.type = "sine";
      o.frequency.setValueAtTime(freq * 1.25, t);
      o.frequency.exponentialRampToValueAtTime(freq * 0.7, t + 3.1);
      depth.connect(o.frequency);
      o.connect(f);
      o.start(t);
      o.stop(t + 3.3);
    }
  },
  // a firework's bang and the crackle after it
  boom(v) {
    const t = ctx.currentTime;
    noiseHit(sfx, t, "lowpass", 700, 0.7, 0.005, 0.5, 0.5 * v, 120);
    for (let i = 0; i < 6; i++) noiseHit(sfx, t + 0.15 + i * rnd(0.05, 0.12), "highpass", 4000, 1, 0.001, 0.04, 0.09 * v);
  },
  hiss(v) {
    const t = ctx.currentTime;
    noiseHit(sfx, t, "highpass", 3500, 0.8, 0.05, 1.6, 0.18 * v, 1200);
  },
  clang(v) {
    // a smith's rhythm: a heavy blow, two lighter, a ringing one
    const t = ctx.currentTime;
    const pn = pan();
    const k = clangN++ % 4;
    v *= [1, 0.55, 0.7, 0.9][k];
    const pitch = [1, 1.12, 0.94, 1.2][k];
    [1, 2.76, 5.4, 8.9].forEach((m, i) => {
      tone(sfx, "sine", 820 * pitch * m, t, 0.002, (k === 3 ? 0.9 : 0.5) / (1 + i * 0.5), (0.09 / (1 + i)) * v, { pan: pn });
    });
    noiseHit(sfx, t, "highpass", 3000, 1, 0.001, 0.05, 0.08 * v);
  },
  // Villager babble: formant-filtered syllables. The style follows the
  // speech bubble the speaker is showing.
  chat(v) {
    const t0 = ctx.currentTime;
    const n = 2 + Math.floor(Math.random() * 5);
    const pitch = voicePitch();
    const pn = pan();
    const T = tm();
    for (let i = 0; i < n; i++) {
      syllable(t0 + i * rnd(0.07, 0.12) * T.len, T.type || "square", [pitch * rnd(0.85, 1.25)], 0.07 * T.len, 0.04 * v, rnd(700, 1600), pn);
    }
  },
  chat_heart(v) {
    // a soft, slow "aww": gentle rise and fall
    const t0 = ctx.currentTime;
    const pitch = voicePitch() * 0.95;
    const pn = pan();
    for (let i = 0; i < 2 + Math.floor(Math.random() * 2); i++) {
      const t = t0 + i * 0.26;
      syllable(t, "triangle", [pitch, pitch * 1.3, pitch * 0.95], 0.22, 0.16 * v, 900, pn);
    }
  },
  chat_laugh(v) {
    // quick "ha-ha-ha" stepping down in pitch
    const t0 = ctx.currentTime;
    const pitch = voicePitch() * 1.3;
    const pn = pan();
    const n = 4 + Math.floor(Math.random() * 3);
    for (let i = 0; i < n; i++) {
      syllable(t0 + i * 0.085, "square", [pitch * (1 - i * 0.05)], 0.05, 0.04 * v, 1400, pn);
      noiseHit(sfx, t0 + i * 0.085, "bandpass", 2400, 2, 0.003, 0.03, 0.02 * v);
    }
  },
  chat_note(v) {
    // sing-song: syllables on a pentatonic tune
    const t0 = ctx.currentTime;
    const base = 56 + 12 * Math.log2(cur.pitch);
    const pn = pan();
    let step = Math.floor(Math.random() * 3);
    for (let i = 0; i < 3 + Math.floor(Math.random() * 3); i++) {
      step = Math.max(0, Math.min(6, step + [-1, 1, 1, 2][Math.floor(Math.random() * 4)]));
      const f = midi(base + PENTA[step % 5] + 12 * Math.floor(step / 5));
      syllable(t0 + i * 0.16, "triangle", [f, f * 1.01], 0.14, 0.11 * v, 1200, pn);
    }
  },
  bark(v) {
    const t = ctx.currentTime;
    const pn = pan();
    for (let i = 0; i < 2; i++) {
      tone(sfx, "sawtooth", 520, t + i * 0.22, 0.01, 0.11, 0.26 * v, {
        to: 260, filter: "lowpass", ff: 1300, pan: pn,
      });
    }
  },
  hoot(v) {
    // a tawny owl: a soft "hoo", a pause, then "hu-hooooo" with a wavering tail
    const t = ctx.currentTime;
    const pn = pan();
    const f = rnd(0.94, 1.06);
    const note = (at, hz, len, peak, to) => {
      tone(sfx, "sine", hz * f, t + at, 0.06, len, 0.2 * peak * v, { to: to * f, pan: pn });
      tone(sfx, "sine", hz * 2 * f, t + at, 0.06, len * 0.8, 0.07 * peak * v, { to: to * 2 * f, pan: pn });
    };
    note(0, 440, 0.4, 0.8, 410);
    note(0.8, 430, 0.16, 0.7, 430);
    note(1.05, 420, 1.0, 1.0, 340);
  },
  gull(v) {
    // a herring gull: a few harsh, falling "kyow" cries
    const t = ctx.currentTime;
    const pn = pan();
    const f = rnd(0.9, 1.15);
    const n = 2 + Math.floor(Math.random() * 3);
    for (let i = 0; i < n; i++) {
      tone(sfx, "sawtooth", 1500 * f, t + i * 0.3, 0.02, 0.22, 0.07 * v, {
        to: 800 * f, filter: "bandpass", ff: 1800, q: 3, pan: pn,
      });
    }
  },
  howl(v) {
    // a wolf: a long rising note that sags and fades away
    const t = ctx.currentTime;
    const pn = pan();
    const f = rnd(0.9, 1.1);
    tone(sfx, "sine", 300 * f, t, 0.4, 2.4, 0.16 * v, { to: 520 * f, pan: pn });
    tone(sfx, "sine", 520 * f, t + 1.4, 0.05, 1.8, 0.14 * v, { to: 330 * f, pan: pn });
    tone(sfx, "triangle", 600 * f, t, 0.5, 2.6, 0.04 * v, { to: 1000 * f, filter: "lowpass", ff: 1800, pan: pn });
  },
  meow(v) {
    // a rising "mee" and a falling "ow"
    const t = ctx.currentTime;
    const pn = pan();
    const f = rnd(0.9, 1.15);
    tone(sfx, "sawtooth", 520 * f, t, 0.03, 0.2, 0.2 * v, { to: 820 * f, filter: "bandpass", ff: 1500, q: 2, pan: pn });
    tone(sfx, "sawtooth", 820 * f, t + 0.2, 0.01, 0.3, 0.18 * v, { to: 480 * f, filter: "bandpass", ff: 1200, q: 2, pan: pn });
  },
  cluck(v) {
    const t = ctx.currentTime;
    const pn = pan();
    for (let i = 0; i < 3; i++) {
      tone(sfx, "square", 820, t + i * 0.09, 0.005, 0.06, 0.12 * v, {
        to: 560, filter: "bandpass", ff: 950, q: 1, pan: pn,
      });
    }
  },
  baa(v) {
    const t = ctx.currentTime;
    const o = tone(sfx, "sawtooth", 230, t, 0.05, 0.6, 0.3 * v, {
      filter: "bandpass", ff: 900, q: 2, pan: pan(),
    });
    const lfo = ctx.createOscillator();
    lfo.frequency.value = 22;
    const lg = ctx.createGain();
    lg.gain.value = 12;
    lfo.connect(lg).connect(o.frequency);
    lfo.start(t);
    lfo.stop(t + 0.7);
  },
  moo(v) {
    const t = ctx.currentTime;
    tone(sfx, "sawtooth", 140, t, 0.2, 1.0, 0.06 * v, {
      to: 110, filter: "lowpass", ff: 600, pan: pan(),
    });
  },
  splash(v) {
    noiseHit(sfx, ctx.currentTime, "bandpass", 1800, 1, 0.01, 0.25, 0.45 * v, 600);
  },
  // A semaphore arm moving: the wire rattling over its pulleys, the arm
  // swinging, and the clunk of the counterweight. Raising to clear is a
  // touch higher than dropping back to danger.
  semaphore(v) {
    const t = ctx.currentTime;
    const pn = pan();
    const up = cur.pitch >= 1;
    // the wire rattling over its pulleys
    for (let i = 0; i < 7; i++) {
      noiseHit(sfx, t + i * 0.03 + rnd(0, 0.01), "bandpass", rnd(2000, 3200), 5, 0.001, 0.035, 0.09 * v);
    }
    // the arm swinging on its spindle: a short wooden creak
    tone(sfx, "sawtooth", up ? 210 : 260, t + 0.08, 0.02, 0.16, 0.05 * v, { to: up ? 300 : 180, pan: pn });
    // the counterweight landing: a solid clunk with a ring of iron
    const at = t + 0.26;
    tone(sfx, "triangle", up ? 150 : 115, at, 0.003, 0.22, 0.5 * v, { to: up ? 80 : 60, pan: pn });
    noiseHit(sfx, at, "lowpass", 900, 1, 0.002, 0.09, 0.3 * v);
    [1, 2.4, 4.1].forEach((m, i) => {
      tone(sfx, "sine", (up ? 660 : 590) * m, at, 0.002, 0.3 / (1 + i), (0.07 / (1 + i)) * v, { pan: pn });
    });
  },
  door(v) {
    const t = ctx.currentTime;
    tone(sfx, "triangle", 160, t, 0.005, 0.12, 0.15 * v, { to: 90 });
  },
  crier(v) {
    // a handbell, then a call: "Oyez! Market's open!"
    const t = ctx.currentTime;
    const pn = pan();
    for (let i = 0; i < 3; i++) {
      tone(sfx, "sine", 1380, t + i * 0.22, 0.002, 0.3, 0.07 * v, { pan: pn });
      tone(sfx, "sine", 2760, t + i * 0.22, 0.002, 0.15, 0.03 * v, { pan: pn });
    }
    const pitch = voicePitch();
    for (let i = 0; i < 6; i++) {
      syllable(t + 0.8 + i * 0.16, "sawtooth", [pitch * (i % 3 === 0 ? 1.2 : 1), pitch], 0.13, 0.05 * v, 900, pn);
    }
  },
  quack(v) {
    const t = ctx.currentTime;
    const pn = pan();
    for (let i = 0; i < 2 + Math.floor(Math.random() * 2); i++) {
      tone(sfx, "sawtooth", rnd(320, 380), t + i * 0.16, 0.01, 0.1, 0.22 * v, {
        to: 260, filter: "bandpass", ff: 1100, q: 3, pan: pn,
      });
    }
  },
  sheepbell(v) {
    const t = ctx.currentTime;
    const pn = pan();
    for (let i = 0; i < 1 + Math.floor(Math.random() * 3); i++) {
      const s = t + i * rnd(0.12, 0.3);
      tone(sfx, "sine", 2150, s, 0.002, 0.5, 0.05 * v, { pan: pn });
      tone(sfx, "sine", 3420, s, 0.002, 0.25, 0.025 * v, { pan: pn });
    }
  },
  woodpecker(v) {
    const t = ctx.currentTime;
    const n = 10 + Math.floor(Math.random() * 8);
    for (let i = 0; i < n; i++) {
      const at = t + i * 0.055, k = v * (1 - i / n * 0.5);
      noiseHit(sfx, at, "bandpass", 1500, 2, 0.001, 0.03, 0.7 * k);
      tone(sfx, "square", 900, at, 0.001, 0.02, 0.1 * k, { to: 500, filter: "bandpass", ff: 1400, q: 2 });
    }
  },
  clop(v) {
    const t = ctx.currentTime;
    tone(sfx, "triangle", rnd(380, 460), t, 0.002, 0.06, 0.35 * v, { to: 200, filter: "bandpass", ff: 900, q: 2 });
    noiseHit(sfx, t, "bandpass", 1200, 2, 0.001, 0.03, 0.3 * v);
  },
  whistle(v) {
    const t = ctx.currentTime;
    tone(sfx, "sine", 1800, t, 0.02, 0.5, 0.15 * v, { to: 2400, glide: 0.3 });
  },
};
