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
};

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
