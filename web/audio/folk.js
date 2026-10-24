// Meeple soundscape, The evening gathering: a jig on the fiddle and the choir.
// One of the `web/audio/*.js` scripts, loaded in order as classic scripts so they
// share one global scope (see docs/audio.md).

// ---------- evening gathering: a jig ----------
// 6/8 time, two bars per chord through I IV I V, scheduled a little ahead.
// A fiddle-ish lead wanders over chord tones, with bass on the main beats,
// a tambourine, and "la-la" voices when people are singing.

const FOLK_LEVEL = 0.9;
const JIG_EIGHTH = 0.19;
const JIG_CHORDS = [[0, 4, 7], [5, 9, 12], [0, 4, 7], [7, 11, 14]];
let jigStep = 0;
let jigNote = 4;

function jigTick(t) {
  // in the same key as the music playing (a fifth above its root)
  const JIG_ROOT = ROOT + 7;
  if (next.jig < t) next.jig = t + 0.05;
  while (next.jig < t + 0.3) {
    const at = next.jig;
    const beat = jigStep % 6;
    const bar = Math.floor(jigStep / 6);
    const chord = JIG_CHORDS[Math.floor(bar / 2) % JIG_CHORDS.length];
    const band = p[P.FESTIVAL];
    if (band > 0.02) {
      if (beat === 0 || beat === 3) {
        const root = midi(JIG_ROOT - 12 + chord[beat === 0 ? 0 : 2]);
        tone(folk, "triangle", root, at, 0.005, 0.32, 0.16);
        noiseHit(folk, at, "highpass", 6500, 0.8, 0.002, 0.09, beat === 0 ? 0.09 : 0.06);
        // a bodhran under it, and the second fiddle a third above on the downbeat
        tone(folk, "sine", 120, at, 0.004, 0.2, beat === 0 ? 0.14 : 0.09, { to: 65 });
        if (beat === 0 && band > 0.5) fiddle(midi(JIG_ROOT + 12 + chord[1]), at, JIG_EIGHTH * 2, folk);
      } else {
        if (beat === 4) noiseHit(folk, at, "bandpass", 1500, 1, 0.002, 0.05, 0.03);
        noiseHit(folk, at, "highpass", 8000, 0.8, 0.002, 0.04, 0.025);
      }
      // the lead: mostly on every eighth, sometimes holding a note
      if (beat === 0 || Math.random() < 0.82) {
        jigNote = Math.max(0, Math.min(9, jigNote + [-2, -1, -1, 0, 1, 1, 2][Math.floor(Math.random() * 7)]));
        const scale = [0, 2, 4, 5, 7, 9, 11, 12, 14, 16];
        let n = JIG_ROOT + 12 + scale[jigNote];
        // land on a chord tone on the strong beats
        if (beat === 0 || beat === 3) n = JIG_ROOT + 12 + chord[Math.floor(Math.random() * 3)];
        fiddle(midi(n), at, JIG_EIGHTH * (Math.random() < 0.2 ? 2 : 1));
      }
    }
    if (beat === 0 && bar % 2 === 0 && p[P.CHOIR] > 0.02) {
      chord.forEach((iv) => choir(midi(JIG_ROOT + iv), at, JIG_EIGHTH * 12, Math.min(p[P.CHOIR], 0.67), folk));
    }
    jigStep++;
    next.jig += JIG_EIGHTH;
  }
}

/// `soft` gives a mellower, slower-bowed tone for background tunes; a number
/// sets its lowpass cutoff in Hz (lower is duller).
function fiddle(freq, t, len, dest, soft = false) {
  const o = ctx.createOscillator();
  o.type = "sawtooth";
  o.frequency.value = freq;
  const vib = ctx.createOscillator();
  vib.frequency.value = 5.5;
  const vg = ctx.createGain();
  vg.gain.value = freq * (soft ? 0.004 : 0.006);
  vib.connect(vg).connect(o.frequency);
  const f = ctx.createBiquadFilter();
  f.type = soft ? "lowpass" : "bandpass";
  f.frequency.value = soft === true ? 1500 : soft || 1700;
  f.Q.value = soft ? 0.5 : 0.7;
  const g = ctx.createGain();
  const lvl = soft ? 0.09 : 0.11;
  g.gain.setValueAtTime(0.0001, t);
  g.gain.linearRampToValueAtTime(lvl, t + (soft ? 0.2 : 0.03));
  g.gain.setValueAtTime(lvl * 0.9, t + len * 0.75);
  g.gain.linearRampToValueAtTime(0.0001, t + len + 0.05);
  o.connect(f).connect(g).connect(dest || folk);
  for (const x of [o, vib]) {
    x.start(t);
    x.stop(t + len + 0.1);
  }
}

// formants (F1, F2, F3) of the vowels the choirs sing: ah, oh, oo, eh
const VOWELS = [[730, 1090, 2440], [570, 840, 2410], [300, 870, 2240], [530, 1840, 2480]];

/// A sung vowel: detuned saws through formant filters, swelling in, with a
/// vibrato that comes on late and a little breath.
function choir(freq, t, len, level, dest, vowel = 0, vib = 0.6) {
  const lite = p[P.TIER] >= 3;
  const amp = (0.042 * Math.min(2.4, level * 1.5)) / (lite ? 1.2 : 1);
  const g = ctx.createGain();
  g.gain.setValueAtTime(0.0001, t);
  g.gain.linearRampToValueAtTime(amp, t + Math.min(0.5, len * 0.2));
  g.gain.setValueAtTime(amp * 0.9, t + len * 0.7);
  g.gain.linearRampToValueAtTime(0.0001, t + len);
  g.connect(dest || folk);
  const lfo = ctx.createOscillator();
  lfo.frequency.value = 5.2 + mr() * 0.6;
  const lg = ctx.createGain();
  lg.gain.setValueAtTime(0, t);
  lg.gain.linearRampToValueAtTime(freq * 0.006 * vib, t + Math.min(0.9, len * 0.4));
  lfo.connect(lg);
  lfo.start(t);
  lfo.stop(t + len + 0.1);
  const formants = VOWELS[vowel] || VOWELS[0];
  const shape = [[formants[0], 6, 1], [formants[1], 8, 0.6], [formants[2], 10, 0.25]].slice(0, lite ? 2 : 3);
  for (const det of lite ? [-8, 9] : [-11, 0, 10]) {
    const o = ctx.createOscillator();
    o.type = "sawtooth";
    o.frequency.value = freq;
    o.detune.value = det;
    lg.connect(o.frequency);
    for (const [ff, q, a] of shape) {
      const f = ctx.createBiquadFilter();
      f.type = "bandpass";
      f.frequency.value = ff;
      f.Q.value = q;
      const fg = ctx.createGain();
      fg.gain.value = a;
      o.connect(f).connect(fg).connect(g);
    }
    o.start(t);
    o.stop(t + len + 0.1);
  }
  noiseHit(g, t, "bandpass", 2600, 1.5, Math.min(0.4, len * 0.2), len * 0.5, 0.05);
}
