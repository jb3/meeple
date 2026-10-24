// A favicon that follows the game: a meeple dressed for the in-game season,
// with a darker sky at night. The Rust side already reports both to the audio
// engine every frame, so we hang off its per-frame hook.
(() => {
  const MEEPLE = new Path2D(
    "M50 34C42 34 38 39 31 41L9 49C4 51 5 58 10 60L30 62L24 89C23 94 26 96 31 96" +
    "L69 96C74 96 77 94 76 89L70 62L90 60C95 58 96 51 91 49L69 41C62 39 58 34 50 34Z");
  const SEASONS = {
    spring: { sky: ["#bfe8ff", "#e8f7d4"], ground: "#7cc65a", meeple: "#f4b6c8" },
    summer: { sky: ["#4cb4f5", "#bfeaff"], ground: "#4fae3c", meeple: "#ffd23f" },
    autumn: { sky: ["#f7b267", "#fde2b3"], ground: "#c9772b", meeple: "#c8452d" },
    winter: { sky: ["#9fb7d6", "#e6eef7"], ground: "#f4f8fc", meeple: "#3b6fb6" },
  };

  const names = ["spring", "summer", "autumn", "winter"];
  let season = "spring";
  let night = false;
  const occasion = "";

  function draw(ctx, S) {
    const k = S / 100;
    const t = SEASONS[season];
    const rr = (x, y, w, h, r) => { ctx.beginPath(); ctx.roundRect(x, y, w, h, r); };
    ctx.clearRect(0, 0, S, S);
    // sky and ground in a rounded tile
    ctx.save();
    rr(0, 0, S, S, S * 0.22);
    ctx.clip();
    const g = ctx.createLinearGradient(0, 0, 0, S);
    let [a, b] = occasion === "halloween" ? ["#2a1240", "#7a3b1d"] : t.sky;
    if (occasion === "christmas") [a, b] = ["#7a1b24", "#c4343f"];
    g.addColorStop(0, a); g.addColorStop(1, b);
    ctx.fillStyle = g; ctx.fillRect(0, 0, S, S);
    if (night && !occasion) {
      ctx.fillStyle = "rgba(10,16,48,0.62)"; ctx.fillRect(0, 0, S, S);
      ctx.fillStyle = "#fff6c8";
      ctx.beginPath(); ctx.arc(S * 0.8, S * 0.2, S * 0.09, 0, 7); ctx.fill();
      ctx.fillStyle = "#fff";
      for (const [x, y] of [[0.15, 0.15], [0.35, 0.08], [0.6, 0.18]]) ctx.fillRect(x * S, y * S, S * 0.03, S * 0.03);
    } else if (!occasion) {
      // a sun (a pale one in winter)
      ctx.fillStyle = season === "winter" ? "#fffbe8" : "#ffe27a";
      ctx.beginPath(); ctx.arc(S * 0.8, S * 0.2, S * 0.1, 0, 7); ctx.fill();
    }
    ctx.fillStyle = occasion === "halloween" ? "#1b0f26" : occasion === "christmas" ? "#f4f8fc" : t.ground;
    ctx.fillRect(0, S * 0.8, S, S * 0.2);
    ctx.restore();

    // the meeple
    ctx.save();
    ctx.scale(k * 0.82, k * 0.82); ctx.translate(11, 8);
    ctx.fillStyle = occasion === "halloween" ? "#111" : occasion === "christmas" ? "#f4f8fc" : t.meeple;
    ctx.strokeStyle = "rgba(20,20,30,0.85)"; ctx.lineWidth = 6; ctx.lineJoin = "round";
    ctx.beginPath(); ctx.arc(50, 20, 15, 0, 7); ctx.fill(); ctx.stroke();
    ctx.stroke(MEEPLE); ctx.fill(MEEPLE);
    ctx.beginPath(); ctx.arc(50, 20, 12, 0, 7); ctx.fill();

    const accent = (draw) => { ctx.save(); draw(); ctx.restore(); };
    if (occasion === "halloween") {
      accent(() => { // jack-o'-lantern face on a ghost-black body, plus a witch hat
        ctx.fillStyle = "#ff9a1f";
        ctx.beginPath(); ctx.arc(50, 20, 12, 0, 7); ctx.fill();
        ctx.fillStyle = "#111";
        ctx.fillRect(43, 15, 4, 5); ctx.fillRect(53, 15, 4, 5); ctx.fillRect(44, 25, 12, 3);
        ctx.fillStyle = "#4b1d6b";
        ctx.beginPath(); ctx.moveTo(34, 10); ctx.lineTo(50, -18); ctx.lineTo(66, 10); ctx.closePath(); ctx.fill();
        ctx.fillRect(30, 7, 40, 5);
      });
    } else if (occasion === "christmas") {
      accent(() => { // santa hat
        ctx.fillStyle = "#d3202c";
        ctx.beginPath(); ctx.moveTo(36, 12); ctx.lineTo(54, -14); ctx.lineTo(66, 12); ctx.closePath(); ctx.fill();
        ctx.fillStyle = "#fff";
        ctx.fillRect(33, 8, 36, 7);
        ctx.beginPath(); ctx.arc(54, -14, 5, 0, 7); ctx.fill();
      });
    } else if (season === "winter") {
      accent(() => { ctx.fillStyle = "#fff"; ctx.beginPath(); ctx.ellipse(50, 8, 14, 6, 0, 0, 7); ctx.fill(); });
    } else if (season === "spring") {
      accent(() => {
        ctx.fillStyle = "#ff6fa5";
        for (const [x, y] of [[40, 8], [52, 4], [62, 12]]) { ctx.beginPath(); ctx.arc(x, y, 5, 0, 7); ctx.fill(); }
      });
    } else if (season === "autumn") {
      accent(() => {
        ctx.fillStyle = "#e8891f";
        ctx.beginPath(); ctx.moveTo(50, -2); ctx.quadraticCurveTo(66, 6, 52, 14); ctx.quadraticCurveTo(36, 6, 50, -2); ctx.fill();
      });
    }
    ctx.restore();

    // winter ground: snow drifts; autumn: fallen leaves
    if (season === "winter" || occasion === "christmas") {
      ctx.fillStyle = "#fff";
      for (const x of [0.12, 0.3, 0.86]) { ctx.beginPath(); ctx.arc(x * S, S * 0.62, S * 0.025, 0, 7); ctx.fill(); }
    }
  }

  let shown = "";
  function apply() {
    const key = season + night;
    if (key === shown) return;
    shown = key;
    const c = document.createElement("canvas");
    c.width = c.height = 64;
    const ctx = c.getContext("2d");
    if (!ctx || !ctx.roundRect) return;
    draw(ctx, 64);
    let link = document.querySelector('link[rel="icon"][data-seasonal]');
    if (!link) {
      document.querySelectorAll('link[rel~="icon"]').forEach((l) => l.remove());
      link = document.createElement("link");
      link.rel = "icon"; link.type = "image/png"; link.dataset.seasonal = "";
      document.head.appendChild(link);
    }
    link.href = c.toDataURL("image/png");
  }

  if (window.meepleAudio) {
    // params[9] is how dark it is, params[10] the season (0 spring .. 3 winter)
    window.meepleAudio.onFrame = (params) => {
      season = names[Math.floor(params[10]) % 4] || season;
      night = params[9] > 0.5;
      apply();
    };
  }
})();
