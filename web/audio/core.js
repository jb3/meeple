// Meeple soundscape: everything here is synthesised with WebAudio, no samples.
//
// The Rust side calls `meepleAudio.update(params, events)` once per frame:
//   params: Float32Array, see P below for indices
//   events: "name:volume;name:volume" one-shot sounds that happened this frame

const P = {
  HOUR: 0, RAIN: 1, WIND: 2, SNOW: 3, CROWD: 4, WATER: 5, NATURE: 6,
  TRAIN: 7, TRAIN_SPEED: 8, NIGHT: 9, SEASON: 10, STORM: 11,
  // an evening gathering nearby: band (dancers + musician) and choir loudness
  FESTIVAL: 12, CHOIR: 13,
  // which place the camera is in (0..1), so each has its own key and motif
  PLACE: 14,
  // how far out the camera is (small = close in)
  ZOOM: 15,
  // the nearest windmill (0..1), its sail angle and where it is across the screen
  MILL: 16, SAIL: 17, MILL_PAN: 18,
  // the nearest waterwheel (0..1)
  WHEEL: 19,
  // 0 plains, 1 desert, 2 cold or high, 3 forest
  BIOME: 20,
  FOG: 21,
  // an event nearby with its own music (1 fire, 2 wedding, 3 christening, 4 closing time), and how near
  CUE: 22, CUE_LEVEL: 23,
  // the inn, children at play, how enclosed the streets are, the church close by
  TAVERN: 24, CHILDREN: 25, ENCLOSED: 26, CHURCH: 27,
  // the renderer's quality tier (0 = best), so the mix can skip what it can't afford
  TIER: 28,
  // the world's seed, so each world keeps its own musical character
  SEED: 29,
};

let ctx = null;
let master, sfx, amb, music, verb, voice, folk, hush, verbLP, verbGain, tavernBus, cueBus;
// the two rooms the sends can ring in, the ducking gain over the music, the train's distance filter
let room, roomG, hall, hallG, duckG, trainBus;
// level of villager babble; one knob for all the chatter styles
const VOICE_LEVEL = 9.0;
// level of the train horn
const HORN_LEVEL = 0.13;
let loops = {};
let noiseBuf = null;
let muted = false;
let musicOn = true;
const p = new Float32Array(32);
let chuffN = 0;
let gust = 0.5, gustTo = 0.5, lastGustT = 0;
// doppler: the train's pitch leans up as it comes closer, down as it leaves
let dop = 1, dopTo = 1, lastTrain = 0, lastTrainT = 0;
const next = { bird: 0, cricket: 0, drip: 0, chug: 0, clack: 0, pump: 0, owl: 0, key: 0, gust: 0, frog: 0, jig: 0, woodpecker: 0, wheel: 0, kids: 0, tav: 0, clink: 0, echo: 0, organ: 0, cue: 0, gull: 0, horn: 0 };
let lastQuarter = null;
let clangN = 0;
let cueStep = 0;
let cueKind = 0;
let lastCue = 0, lastCueLevel = 0;
const smooth = (a, b, x) => {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};

// ---------- setup ----------

function start() {
  // a bigger buffer: TVs and old phones glitch under load with the default
  ctx = new (window.AudioContext || window.webkitAudioContext)({ latencyHint: "playback" });
  master = ctx.createGain();
  // a fixed stereo mix: sources come and go, mono and stereo, and a channel
  // count that changed under the filters downstream could make them glitch
  stereo(master);
  master.gain.value = muted ? 0 : 0.85;
  const comp = ctx.createDynamicsCompressor();
  comp.threshold.value = -18;
  comp.ratio.value = 3;
  // snowfall and fog hush everything: a lowpass the whole mix passes through
  hush = ctx.createBiquadFilter();
  hush.type = "lowpass";
  hush.frequency.value = 20000;
  hush.Q.value = 0.5;
  stereo(hush);
  master.connect(hush).connect(comp).connect(ctx.destination);

  // the room: livelier among buildings and by the church, deadened by woods and snow
  verbLP = ctx.createBiquadFilter();
  verbLP.type = "lowpass";
  verbLP.frequency.value = 7000;
  stereo(verbLP);
  verbGain = ctx.createGain();
  verbGain.gain.value = 0.55;
  verbLP.connect(verbGain).connect(master);
  // every send enters `verb` and rings in a tight room (streets) and a long
  // dark hall (wide open country, fog, the church); the mix crossfades
  verb = stereo(ctx.createGain());
  const lite = p[P.TIER] >= 3;
  room = ctx.createConvolver();
  room.buffer = lite ? impulse(2.4, 2.4) : impulse(1.2, 3.0);
  roomG = ctx.createGain();
  verb.connect(room).connect(roomG).connect(verbLP);
  if (!lite) {
    hall = ctx.createConvolver();
    hall.buffer = impulse(5.5, 1.7, 0.55);
    hallG = ctx.createGain();
    hallG.gain.value = 0;
    verb.connect(hall).connect(hallG).connect(verbLP);
  }

  sfx = bus(0.9, 0.25);
  amb = bus(0.8, 0.1);
  music = bus(musicOn ? MUSIC_LEVEL : 0, 0.45);
  // everything musical passes a ducking gain, so a bell or a thunderclap can dip it
  duckG = stereo(ctx.createGain());
  music.disconnect();
  music.connect(duckG);
  duckG.connect(master);
  duckG.connect(music._send);
  // trains are heard through air: duller the further off they are
  trainBus = stereo(ctx.createBiquadFilter());
  trainBus.type = "lowpass";
  trainBus.frequency.value = 16000;
  trainBus.Q.value = 0.4;
  trainBus.connect(sfx);
  voice = ctx.createGain();
  voice.gain.value = VOICE_LEVEL;
  voice.connect(sfx);
  folk = bus(0, 0.35);
  tavernBus = bus(0, 0.5);
  cueBus = bus(0, 0.45);
  musicSetup();

  noiseBuf = makeNoise(4);
  // a whisper of noise into the music's delay loop, so its feedback tails
  // never decay into denormal numbers (which can make a slow CPU stutter)
  const floor = ctx.createBufferSource();
  floor.buffer = noiseBuf;
  floor.loop = true;
  const floorGain = ctx.createGain();
  floorGain.gain.value = 3e-6;
  floor.connect(floorGain).connect(melodyBus);
  floor.start();

  loops.wind = noiseLoop(amb, "bandpass", 420, 0.6);
  loops.gust = noiseLoop(amb, "lowpass", 260, 0.5);
  loops.rain = noiseLoop(amb, "highpass", 1400, 0.4);
  loops.rainLow = noiseLoop(amb, "lowpass", 900, 0.3);
  loops.water = noiseLoop(amb, "bandpass", 700, 2.5);
  loops.crowd = noiseLoop(amb, "bandpass", 520, 1.8);
  loops.rumble = noiseLoop(trainBus, "lowpass", 110, 0.8);
  loops.hiss = noiseLoop(trainBus, "highpass", 3500, 0.7);
  // the wind hums in the music's key: two very narrow bands retuned with each chord
  loops.humA = noiseLoop(amb, "bandpass", 262, 28);
  loops.humB = noiseLoop(amb, "bandpass", 392, 28);
  // the far-off hum of a whole valley, heard when zoomed out
  loops.distant = noiseLoop(amb, "bandpass", 170, 0.45);
  // the low murmur of streets closed in by buildings
  loops.town = noiseLoop(amb, "bandpass", 300, 0.7);

  document.getElementById("listen")?.classList.add("gone");
}

function stereo(node) {
  node.channelCount = 2;
  node.channelCountMode = "explicit";
  node.channelInterpretation = "speakers";
  return node;
}

function bus(gain, send) {
  const g = stereo(ctx.createGain());
  g.gain.value = gain;
  g.connect(master);
  const s = ctx.createGain();
  s.gain.value = send;
  g.connect(s).connect(verb);
  g._send = s;
  return g;
}

function makeNoise(seconds) {
  const len = ctx.sampleRate * seconds;
  const buf = ctx.createBuffer(1, len, ctx.sampleRate);
  const d = buf.getChannelData(0);
  // pinkish noise (Paul Kellet's economy filter)
  let b0 = 0, b1 = 0, b2 = 0;
  for (let i = 0; i < len; i++) {
    const w = Math.random() * 2 - 1;
    b0 = 0.99765 * b0 + w * 0.099046;
    b1 = 0.963 * b1 + w * 0.2965164;
    b2 = 0.57 * b2 + w * 1.0526913;
    d[i] = (b0 + b1 + b2 + w * 0.1848) * 0.18;
  }
  return buf;
}

/// A decaying burst of noise; `dark` (0..1) smooths away the treble.
function impulse(seconds, decay, dark = 0) {
  const len = Math.floor(ctx.sampleRate * seconds);
  const buf = ctx.createBuffer(2, len, ctx.sampleRate);
  for (let c = 0; c < 2; c++) {
    const d = buf.getChannelData(c);
    let lp = 0;
    for (let i = 0; i < len; i++) {
      lp += ((Math.random() * 2 - 1) - lp) * (1 - dark);
      d[i] = lp * Math.pow(1 - i / len, decay);
    }
  }
  return buf;
}

/// Dip the music (a bell, a thunderclap) and let it swell back.
function duck(depth, hold, release, at) {
  const g = duckG.gain;
  const t = at === undefined ? ctx.currentTime : at;
  g.cancelScheduledValues(t);
  g.setValueAtTime(g.value, t);
  g.linearRampToValueAtTime(1 - depth, t + 0.04);
  g.setValueAtTime(1 - depth, t + 0.04 + hold);
  g.linearRampToValueAtTime(1, t + 0.04 + hold + release);
}

function noiseLoop(dest, type, freq, q) {
  const src = ctx.createBufferSource();
  src.buffer = noiseBuf;
  src.loop = true;
  src.loopStart = Math.random();
  const f = ctx.createBiquadFilter();
  f.type = type;
  f.frequency.value = freq;
  f.Q.value = q;
  const g = ctx.createGain();
  g.gain.value = 0;
  src.connect(f).connect(g).connect(dest);
  src.start(0, Math.random() * 3);
  return { f, g, base: freq };
}

// The scene is sampled every frame, but a parameter only needs telling when
// its target actually moves: re-issuing the same target sixty times a second
// for hours keeps the audio thread busy for nothing.
const lastTarget = new WeakMap();
function set(param, value, tc = 0.6) {
  const was = lastTarget.get(param);
  if (was !== undefined && Math.abs(was - value) <= 1e-4 * (1 + Math.abs(value))) return;
  lastTarget.set(param, value);
  param.setTargetAtTime(value, ctx.currentTime, tc);
}
