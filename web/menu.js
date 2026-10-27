// A menu for touch screens, where the keyboard shortcuts can't be reached.
// Each button presses its key, so the game and the audio both hear it.
// Saving a photograph: the app calls this right after drawing a frame, while
// the canvas still holds it.
window.meepleUi = {
  snap() {
    const c = document.getElementById("meeple");
    const a = document.createElement("a");
    a.download = `meeple-${Date.now()}.png`;
    a.href = c.toDataURL("image/png");
    a.click();
  },

  // Start or stop recording the canvas (and the soundscape, once audio has
  // started) to a video file. Returns whether a recording is now running.
  record() {
    if (rec) {
      rec.stop();
      return false;
    }
    const c = document.getElementById("meeple");
    if (!c.captureStream || typeof MediaRecorder === "undefined") return false;
    const stream = c.captureStream(60);
    for (const t of window.meepleAudio?.captureStream?.()?.getAudioTracks() ?? []) stream.addTrack(t);
    // WebM first: Chrome's H.264 encoder fails on odd canvas sizes and on some
    // hardware at high resolutions, so MP4 is left to browsers with nothing else
    const types = ["video/webm;codecs=vp9,opus", "video/webm;codecs=vp8,opus", "video/webm", "video/mp4;codecs=avc1"];
    const type = types.find((t) => MediaRecorder.isTypeSupported(t));
    if (!type) return false;
    const chunks = [];
    rec = new MediaRecorder(stream, { mimeType: type, videoBitsPerSecond: 12_000_000 });
    rec.ondataavailable = (e) => e.data.size && chunks.push(e.data);
    let failed = false;
    rec.onerror = (e) => {
      failed = true;
      console.error("recording failed", e.error);
      if (rec?.state !== "inactive") rec?.stop();
    };
    rec.onstop = () => {
      rec = null;
      const size = chunks.reduce((n, c) => n + c.size, 0);
      // a recording that died at once leaves a few bytes of header: don't hand that over
      if (failed || size < 20000) {
        const t = document.getElementById("toast");
        if (t) {
          t.textContent = "Recording failed in this browser";
          t.classList.remove("hidden");
          t.classList.add("on");
          setTimeout(() => t.classList.remove("on"), 4000);
        }
        return;
      }
      const a = document.createElement("a");
      a.download = `meeple-${Date.now()}.${type.startsWith("video/mp4") ? "mp4" : "webm"}`;
      a.href = URL.createObjectURL(new Blob(chunks, { type }));
      a.click();
      setTimeout(() => URL.revokeObjectURL(a.href), 60000);
    };
    rec.start(1000);
    return true;
  },
  recording: () => !!rec,
};
let rec = null;

(() => {
  const menu = document.getElementById("menu");
  const panel = document.getElementById("menu-panel");
  const toggle = document.getElementById("menu-toggle");
  const SHOW_FOR = 4000;
  let hideTimer = 0;

  if (matchMedia("(pointer: coarse)").matches) {
    const hint = document.getElementById("listen");
    if (hint) hint.textContent = "tap anywhere for sound · ☰ for controls";
  }
  if (!document.fullscreenEnabled) document.getElementById("menu-fullscreen")?.remove();

  const open = () => !panel.classList.contains("hidden");
  function setOpen(on) {
    panel.classList.toggle("hidden", !on);
    toggle.setAttribute("aria-expanded", String(on));
    reveal();
  }
  // show the button for a while after a touch; an open menu stays up
  function reveal() {
    menu.classList.add("visible");
    clearTimeout(hideTimer);
    hideTimer = setTimeout(() => {
      if (!open()) menu.classList.remove("visible");
    }, SHOW_FOR);
  }

  window.addEventListener("pointerdown", (e) => {
    if (e.pointerType === "mouse") return;
    // tapping the world closes the menu
    if (open() && !menu.contains(e.target)) setOpen(false);
    else reveal();
  });

  // each option's button says what it's currently set to
  const onOff = (b) => (b ? "on" : "off");
  function refresh() {
    const d = panel.dataset;
    const audio = window.meepleAudio?.state?.() ?? {};
    const values = {
      auto: d.auto,
      speed: d.speed,
      weather: d.weather,
      pixel: d.pixel,
      sound: onOff(!audio.muted),
      music: onOff(audio.music !== false),
      news: d.news,
      map: d.map,
      chronicle: d.chronicle,
      photo: d.photo,
      record: d.record,
      hud: d.hud,
      fullscreen: onOff(!!document.fullscreenElement),
    };
    for (const b of panel.querySelectorAll("button[data-from]")) {
      const v = values[b.dataset.from];
      if (v !== undefined) b.textContent = `${b.dataset.label}: ${v}`;
    }
  }
  setInterval(refresh, 400);
  toggle.addEventListener("click", refresh);
  panel.addEventListener("click", () => setTimeout(refresh, 60));

  toggle.addEventListener("click", () => setOpen(!open()));
  panel.addEventListener("click", (e) => {
    const key = e.target.closest("button[data-key]")?.dataset.key;
    if (key === undefined) return;
    window.dispatchEvent(new KeyboardEvent("keydown", { key }));
    window.dispatchEvent(new KeyboardEvent("keyup", { key }));
    reveal();
  });
})();
