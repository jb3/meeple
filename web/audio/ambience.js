// Meeple soundscape, Scheduled ambience: birds, crickets, drips, frogs and the other things that happen on their own timers.
// One of the `web/audio/*.js` scripts, loaded in order as classic scripts so they
// share one global scope (see docs/audio.md).

// ---------- scheduled ambience ----------

function scheduled() {
  const t = ctx.currentTime;
  const night = p[P.NIGHT];
  const rain = p[P.RAIN];
  const nature = p[P.NATURE];
  const season = Math.floor(p[P.SEASON]) % 4;
  const day = 1 - night;

  // birdsong by day, fewer in rain and winter, and a dawn chorus that swells
  // as the sun comes up
  const hr = p[P.HOUR];
  const dawn = Math.max(0, 1 - Math.abs(hr - 6.6) / 1.7);
  const birdRate = nature * Math.max(day, dawn * 0.8) * (1 - rain * 0.9) * (season === 3 ? 0.3 : 1) * (1 + dawn * 2.2);
  if (t > next.bird) {
    if (birdRate > 0.05) chirp(birdRate);
    next.bird = t + rnd(0.4, 3.5) / Math.max(birdRate, 0.15);
  }

  // crickets on warm nights
  const cricketRate = nature * night * (season === 1 ? 1 : season === 3 ? 0 : 0.6) * (1 - rain);
  if (t > next.cricket) {
    if (cricketRate > 0.05) cricket(cricketRate);
    next.cricket = t + rnd(0.15, 0.6);
  }

  // frogs near water on spring/summer nights
  if (t > next.frog) {
    const fr = p[P.WATER] * night * (season <= 1 ? 1 : 0.2);
    if (fr > 0.15) tone(amb, "square", rnd(160, 220), t, 0.01, 0.12, 0.03 * fr, { filter: "lowpass", ff: 700, pan: pan() });
    next.frog = t + rnd(0.6, 2.5);
  }

  // a woodpecker drumming in the woods now and then
  if (t > next.woodpecker) {
    if (nature > 0.3 && day > 0.6 && season !== 3 && rain < 0.3) voices.woodpecker(0.6 * nature);
    next.woodpecker = t + rnd(25, 70);
  }

  // gulls over the water by day, and a foghorn when the fog's in
  if (t > next.gull) {
    if (p[P.WATER] > 0.35 && day > 0.4 && rain < 0.5) voices.gull(0.5 + p[P.WATER] * 0.4);
    next.gull = t + rnd(7, 20);
  }
  if (t > next.horn) {
    if (p[P.FOG] > 0.5 && p[P.WATER] > 0.2) voices.foghorn(0.4 + p[P.FOG] * 0.4);
    next.horn = t + rnd(22, 38);
  }

  // a waterwheel turning: paddles slapping the race
  if (t > next.wheel) {
    const wheel = p[P.WHEEL];
    if (wheel > 0.05) {
      noiseHit(amb, t, "bandpass", 900, 1.2, 0.01, 0.2, 0.25 * wheel);
      tone(amb, "triangle", rnd(110, 140), t, 0.002, 0.07, 0.14 * wheel, { to: 70 });
    }
    next.wheel = t + rnd(0.7, 0.95);
  }

  // children playing out
  if (t > next.kids) {
    const kids = p[P.CHILDREN];
    if (kids > 0.05 && day > 0.4 && rain < 0.5) voices.giggle(1.0 + kids * 0.5);
    next.kids = t + rnd(1.5, 5) / Math.max(kids, 0.2);
  }

  // the inn: mugs and cheers through the wall
  set(tavernBus.gain, 1, 0.5);
  if (t > next.clink) {
    const tav = p[P.TAVERN];
    if (tav > 0.08) {
      tone(tavernBus, "sine", rnd(2000, 2900), t, 0.002, 0.25, 0.1 * tav, { pan: rnd(-0.5, 0.5) });
      tone(tavernBus, "sine", rnd(3200, 4200), t, 0.002, 0.12, 0.05 * tav);
      if (Math.random() < 0.3) voices.cheer(0.5 * tav);
    }
    next.clink = t + rnd(0.8, 2.6);
  }

  // closed-in streets: doors echoing between the walls
  set(loops.town.g.gain, p[P.ENCLOSED] * 0.08, 1.5);
  if (t > next.echo) {
    const enc = p[P.ENCLOSED];
    if (enc > 0.2 && day > 0.3) {
      voices.door(0.5 * enc);
    }
    next.echo = t + rnd(0.5, 2.2);
  }

  // by the church: a soft organ chord now and then, swelling and fading in the stone
  if (t > next.organ) {
    const ch = p[P.CHURCH];
    if (ch > 0.1) {
      const root = keyed(130.8);
      [1, 1.25, 1.5, 2].forEach((m, i) => {
        const f = keyed(root * m);
        tone(sfx, "sine", f, t, 2.0, 4.5, 0.09 * ch / (1 + i * 0.4), {});
        tone(sfx, "sine", f * 2, t, 2.2, 4.0, 0.04 * ch / (1 + i * 0.4), {});
      });
    }
    next.organ = t + rnd(9, 16);
  }

  // owls, occasionally
  if (t > next.owl) {
    if (night > 0.7 && nature > 0.2 && Math.random() < 0.7) {
      // a far-off owl: the same call, quieter
      voices.hoot(0.55);
    }
    next.owl = t + rnd(8, 22);
  }

  // drips under rain
  if (t > next.drip) {
    if (rain > 0.15) noiseHit(amb, t, "bandpass", rnd(2500, 5000), 8, 0.001, 0.04, 0.02 * rain);
    next.drip = t + rnd(0.03, 0.25) / Math.max(rain, 0.1);
  }

  // the steam train: chuffs in fours, the first of each strongest, faster
  // as it picks up speed; wheels drumming over the rail joints; and a
  // simmer and the pump's thump while it stands
  const train = p[P.TRAIN];
  const sp = p[P.TRAIN_SPEED];
  if (t > next.chug) {
    if (train > 0.03 && sp > 0.05) {
      chuffN = (chuffN + 1) % 4;
      const accent = chuffN === 0 ? 1 : 0.6;
      // the exhaust: a breathy "tchoo" falling in pitch, over a body thump
      noiseHit(trainBus, t, "bandpass", 1500 * dop, 1.4, 0.004, 0.16, 0.3 * train * accent, 600 * dop);
      noiseHit(trainBus, t, "lowpass", 260 * dop, 1.5, 0.004, 0.1, 0.32 * train * accent);
    }
    next.chug = t + 1 / (1.6 + sp * 6.5);
  }
  if (t > next.clack) {
    if (train > 0.05 && sp > 0.15) {
      // two bogies over each joint: da-dum ... da-dum
      const gap = Math.min(0.22, 0.09 / sp);
      [0, gap, gap * 3.2, gap * 4.2].forEach((dt, i) => {
        const at = t + dt;
        noiseHit(trainBus, at, "bandpass", (2600 + (i % 2) * 500) * dop, 3, 0.001, 0.035, 0.12 * train);
        tone(trainBus, "triangle", (i % 2 ? 95 : 120) * dop, at, 0.002, 0.06, 0.12 * train);
      });
    }
    next.clack = t + 1.1 / Math.max(sp, 0.15);
  }
  loops.hiss && set(loops.hiss.g.gain, train > 0.03 && sp < 0.05 ? train * 0.09 : train * 0.015, 0.4);
  if (t > next.pump) {
    if (train > 0.05 && sp < 0.05) {
      // the air pump at rest: a double thump
      tone(trainBus, "sine", 70, t, 0.01, 0.12, 0.18 * train, { to: 55 });
      tone(trainBus, "sine", 66, t + 0.32, 0.01, 0.12, 0.14 * train, { to: 52 });
    }
    next.pump = t + rnd(1.4, 2.2);
  }

  if (musicOn) musicTick(t);
}

function chirp(v) {
  const t = ctx.currentTime;
  const pn = pan();
  const n = 1 + Math.floor(Math.random() * 5);
  const base = rnd(2200, 4200);
  for (let i = 0; i < n; i++) {
    const s = t + i * rnd(0.06, 0.14);
    tone(amb, "sine", base * rnd(0.9, 1.1), s, 0.005, rnd(0.04, 0.1), 0.09 * Math.min(v, 1), {
      to: base * rnd(1.1, 1.5), pan: pn,
    });
  }
}

function cricket(v) {
  const t = ctx.currentTime;
  const pn = pan();
  const f = rnd(4200, 4800);
  for (let i = 0; i < 3; i++) {
    tone(amb, "sine", f, t + i * 0.035, 0.003, 0.02, 0.012 * v, { pan: pn });
  }
}
