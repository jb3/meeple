// Pixel-art splash: drawn at 1 art-pixel per canvas pixel and scaled up with
// whole-number CSS zoom. It shows from first paint, holds for a moment, then
// leaves once the app reports its first frame (window.__meepleReady).
(() => {
  const el = document.getElementById("splash");
  const cv = document.getElementById("splash-canvas");
  if (!el || !cv) return;

  const MIN_MS = 2400;
  const MAX_MS = 20000;
  const W = 40;
  const H = 36;

  const BIG = {
    M: ["10001", "11011", "10101", "10101", "10001", "10001", "10001"],
    E: ["11111", "10000", "10000", "11110", "10000", "10000", "11111"],
    P: ["11110", "10001", "10001", "11110", "10000", "10000", "10000"],
    L: ["10000", "10000", "10000", "10000", "10000", "10000", "11111"],
  };
  const SMALL = {
    b: ["100", "100", "110", "101", "110"],
    y: ["101", "101", "111", "001", "110"],
    J: ["001", "001", "001", "101", "010"],
    o: ["000", "010", "101", "101", "010"],
    e: ["000", "011", "111", "100", "011"],
  };
  const MEEPLE = [
    "....rrr....",
    "...rrrrr...",
    "...rrhrr...",
    "....rrr....",
    "rrrrrrrrrrr",
    "rhrrrrrrrrr",
    ".rrrrrrrrr.",
    "..rrrrrrr..",
    "..rrrrrrr..",
    ".rrrr.rrrr.",
    ".rrr...rrr.",
  ];
  const SMILEY = [
    "..yyy..",
    ".yyyyy.",
    "yykykyy",
    "yyyyyyy",
    "ykyyyky",
    ".ykkky.",
    "..yyy..",
  ];
  const PAL = {
    r: "#e5533d", h: "#ff9a7a", y: "#ffd23f", k: "#3a2418",
    gold: "#ffd98a", hi: "#fff2c4", shade: "#c9863a", edge: "#3a2418", cream: "#fdf6e3",
  };

  const ctx = cv.getContext("2d");
  cv.width = W;
  cv.height = H;

  const plot = (x, y, c) => {
    ctx.fillStyle = c;
    ctx.fillRect(Math.round(x), Math.round(y), 1, 1);
  };
  const sprite = (rows, ox, oy, colour) => {
    rows.forEach((row, y) => {
      for (let x = 0; x < row.length; x++) {
        const ch = row[x];
        if (ch === "." || ch === "0") continue;
        plot(ox + x, oy + y, colour ? colour(ch, x, y) : PAL[ch] || PAL.cream);
      }
    });
  };
  const word = (font, text, gap) =>
    [...text].reduce((w, ch) => w + (font[ch][0].length) + gap, -gap);

  const start = performance.now();
  let shown = false;

  function draw(t) {
    ctx.clearRect(0, 0, W, H);

    // the meeple hops once the title is down
    const hop = t > 1000 && t < 1600 ? ([0, 2, 3, 3, 2, 0][Math.floor((t - 1000) / 100)] || 0) : 0;
    sprite(MEEPLE, Math.floor((W - 11) / 2), 3 - hop);

    // the title, letter by letter, each dropping into place
    const title = "MEEPLE";
    const tw = word(BIG, title, 2);
    let x = Math.floor((W - tw) / 2);
    const y0 = 17;
    [...title].forEach((ch, i) => {
      const age = t - 300 - i * 140;
      if (age > 0) {
        const drop = age < 280 ? Math.max(0, 6 - Math.floor(age / 40)) : 0;
        const rows = BIG[ch];
        sprite(rows, x + 1, y0 + 1 - drop, () => PAL.edge);
        sprite(rows, x, y0 - drop, (_, cx, cy) => (cy === 0 ? PAL.hi : cy >= 5 ? PAL.shade : PAL.gold));
      }
      x += BIG[ch][0].length + 2;
    });

    // the subline, then the smile
    if (t > 1300) {
      const by = y0 + 11;
      const sw = word(SMALL, "by", 1) + 3 + word(SMALL, "Joe", 1) + 2 + 7;
      let sx = Math.floor((W - sw) / 2);
      for (const part of ["by", "Joe"]) {
        for (const ch of part) {
          sprite(SMALL[ch], sx, by, () => PAL.cream);
          sx += SMALL[ch][0].length + 1;
        }
        sx += 2;
      }
      if (t > 1600) sprite(SMILEY, sx, by - 1);
    }
  }

  function leave() {
    if (shown) return;
    shown = true;
    el.classList.add("out");
    setTimeout(() => el.remove(), 1000);
  }

  function frame(now) {
    if (!el.isConnected) return;
    const t = now - start;
    draw(t);
    if ((window.__meepleReady && t >= MIN_MS) || t >= MAX_MS) leave();
    if (!shown) requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
  el.addEventListener("click", () => window.__meepleReady && leave());
})();
