// Meeple soundscape, Per-frame update from Rust, user controls and the public `meepleAudio` API.
// One of the `web/audio/*.js` scripts, loaded in order as classic scripts so they
// share one global scope (see docs/audio.md).

// ---------- per-frame update from Rust ----------

function update(params, events) {
  if (window.meepleAudio.onFrame) window.meepleAudio.onFrame(params);
  for (let i = 0; i < params.length && i < p.length; i++) p[i] = params[i];
  if (!ctx || ctx.state !== "running") return;

  const wind = p[P.WIND];
  const rain = p[P.RAIN];
  const storm = p[P.STORM];
  const snow = p[P.SNOW];
  const t = ctx.currentTime;

  // gusts come and go in swells that wander at random, not on a fixed sine
  if (t > next.gust) {
    gustTo = Math.random();
    next.gust = t + 1.5 + Math.random() * 4;
  }
  gust += (gustTo - gust) * (1 - Math.exp(-Math.min(0.2, t - lastGustT) / 1.4));
  lastGustT = t;
  set(loops.wind.g.gain, 0.05 + wind * 0.22 * (0.75 + 0.5 * gust) + storm * 0.12 + snow * 0.05, 1.5);
  loops.wind.f.frequency.setTargetAtTime(loops.wind.base * (0.65 + 0.7 * gust), t, 0.6);
  set(loops.gust.g.gain, wind * 0.16 * (0.2 + 0.9 * gust), 1.2);
  const hum = (wind * 0.55 + storm * 0.2) * (0.3 + 0.7 * gust);
  set(loops.humA.g.gain, hum, 1.5);
  set(loops.humB.g.gain, hum * 0.7, 1.5);
  set(loops.rain.g.gain, rain * 0.06, 1.5);
  set(loops.rainLow.g.gain, rain * 0.05 + storm * 0.1, 1.5);
  set(loops.water.g.gain, p[P.WATER] * 0.09, 1.0);
  loops.water.f.frequency.setTargetAtTime(600 + 300 * Math.sin(t * 0.7), t, 0.4);
  set(loops.crowd.g.gain, Math.min(p[P.CROWD], 1) * 0.11 + p[P.TAVERN] * 0.1, 1.0);
  loops.crowd.f.frequency.setTargetAtTime(450 + 180 * Math.sin(t * 2.3) * Math.sin(t * 1.1), t, 0.2);
  set(loops.rumble.g.gain, p[P.TRAIN] * 0.3 * (0.3 + p[P.TRAIN_SPEED]), 0.5);
  if (t - lastTrainT > 0.05) {
    const rate = (p[P.TRAIN] - lastTrain) / (t - lastTrainT);
    lastTrain = p[P.TRAIN];
    lastTrainT = t;
    dopTo = 1 + clampN(rate * 0.25, -0.07, 0.07);
  }
  dop += (dopTo - dop) * 0.1;
  loops.rumble.f.frequency.setTargetAtTime(110 * dop, t, 0.3);
  set(trainBus.frequency, 700 + 15000 * Math.pow(Math.min(p[P.TRAIN], 1), 1.2), 0.5);

  // the further out the camera, the further the mix goes from close detail
  // (voices, footsteps, doors) to the wide wash of the whole valley
  const far = smooth(28, 110, p[P.ZOOM]);
  set(sfx.gain, 0.9 * (1 - far * 0.5), 1.0);
  set(sfx._send.gain, 0.25 + far * 0.35, 1.0);
  set(voice.gain, VOICE_LEVEL * (1 - far * 0.85), 0.6);
  set(amb.gain, 0.8 * (1 + far * 0.25), 1.0);
  set(loops.distant.g.gain, far * (0.06 + Math.min(p[P.CROWD], 1) * 0.04 + wind * 0.04), 1.5);

  // snow and fog muffle the world; woods and snow deaden the room, streets and
  // the church make it ring
  set(hush.frequency, 20000 * Math.pow(0.12, Math.min(1, snow * 0.9 + p[P.FOG] * 0.35)), 2.0);
  set(verbGain.gain, 0.55 + 0.3 * p[P.ENCLOSED] + 0.5 * p[P.CHURCH] - 0.15 * p[P.NATURE], 2.0);
  set(verbLP.frequency, 7000 * (1 - p[P.NATURE] * 0.45) * (1 - snow * 0.4), 2.0);
  // tight in the streets, a long dark hall out in the open, in fog or by the church
  const open = clampN(far * 0.8 + p[P.FOG] * 0.4 + snow * 0.2 + p[P.CHURCH] * 0.6 + (Math.round(p[P.CUE]) === 3 ? p[P.CUE_LEVEL] * 0.5 : 0) - p[P.ENCLOSED] * 0.9, 0, 1);
  set(roomG.gain, hall ? 1 - open * 0.75 : 1, 2.0);
  if (hall) set(hallG.gain, open * 1.1, 2.0);

  // a windmill: a soft whoosh as each sail passes the tower, and the creak of its wood
  const mill = p[P.MILL];
  if (mill > 0.05) {
    const q = Math.floor(p[P.SAIL] / (Math.PI / 2));
    if (lastQuarter !== null && q !== lastQuarter) {
      const pn = p[P.MILL_PAN];
      noiseHit(sfx, t, "bandpass", 520, 0.8, 0.14, 0.34, 0.08 * mill, 260, pn);
      if (q % 2 === 0) tone(sfx, "sawtooth", rnd(80, 95), t, 0.2, 0.55, 0.014 * mill, { to: 66, filter: "lowpass", ff: 300, pan: pn });
    }
    lastQuarter = q;
  } else {
    lastQuarter = null;
  }

  // ambience leads: the music steps back when the scene gets busy
  const fest = p[P.FESTIVAL];
  set(folk.gain, Math.min(1, fest * 1.2) * FOLK_LEVEL, 1.5);
  if (fest > 0.02 || p[P.CHOIR] > 0.02) jigTick(t);
  else next.jig = 0;
  // the music of whatever's going on nearby
  const hourNow = p[P.HOUR];
  const cue = Math.round(p[P.CUE]);
  const cueLevel = cue > 0 ? p[P.CUE_LEVEL] : 0;
  rhythmGate = 1 - Math.min(1, cueLevel * 1.3 + fest * 1.3);
  // a christening is the choir's moment: louder, and ringing in the hall
  set(cueBus.gain, cueLevel * (cue === 3 ? 1.3 : 0.9), 1.2);
  set(cueBus._send.gain, cue === 3 ? 0.95 : 0.45, 1.5);
  // a wedding cut off at its height (the party's wound up): the tune lands on a
  // last chord and bell, and the ordinary music comes back in under it
  if (lastCue === 2 && lastCueLevel > 0.3 && !(cue === 2 && cueLevel > 0.05)) weddingFinale(t);
  lastCue = cue;
  lastCueLevel = cueLevel;
  if (cue > 0 && cueLevel > 0.03) cueTick(t, cue);
  else cueKind = 0;
  if (musicOn) {
    const busy = Math.min(1, Math.min(p[P.CROWD], 1) * 0.6 + p[P.TRAIN] + rain * 0.5 + storm * 0.5 + fest * 1.5 + cueLevel * 0.9);
    set(music.gain, MUSIC_LEVEL * (1 - busy * (0.45 + 0.2 * cueLevel)), 2.0);
  } else if (t > next.key) {
    // with the music off the key still follows the place, for bells and events
    ROOT = keyNow();
    next.key = t + 10;
  }

  if (events) {
    for (const e of events.split(";")) {
      if (!e) continue;
      const [name, vol, pn, pitch] = e.split(":");
      const v = Math.min(parseFloat(vol) || 0, 1.5);
      cur.pan = pn === undefined ? null : parseFloat(pn);
      // the trade rides along in the hundreds of the pitch (see Rust)
      const pv = parseFloat(pitch) || 1;
      cur.role = pv >= 100 ? Math.floor(pv / 100) : 0;
      cur.pitch = pv - cur.role * 100 || 1;
      cur.vol = v;
      if (v > 0.01 && voices[name]) voices[name](v);
      cur.vol = null;
      cur.pan = null;
      cur.pitch = 1;
      cur.role = 0;
    }
  }

  scheduled();
}

// ---------- user controls ----------

function resume() {
  if (!ctx) start();
  else if (ctx.state === "suspended") ctx.resume();
}
window.addEventListener("pointerdown", resume);
// a touch only counts as a user gesture once the finger lifts
window.addEventListener("pointerup", resume);
window.addEventListener("keydown", (e) => {
  resume();
  if (e.key === "m" || e.key === "M") {
    muted = !muted;
    if (master) set(master.gain, muted ? 0 : 0.85, 0.1);
  } else if (e.key === "b" || e.key === "B") {
    musicOn = !musicOn;
    if (music) set(music.gain, musicOn ? MUSIC_LEVEL : 0, 0.3);
  } else if (e.key === "f" || e.key === "F") {
    if (!document.fullscreenElement) document.documentElement.requestFullscreen?.();
    else document.exitFullscreen?.();
  }
});

window.meepleAudio = { update, P, voices: Object.keys(voices), onFrame: null, state: () => ({ muted, music: musicOn }), captureStream: () => capture?.stream ?? null };
