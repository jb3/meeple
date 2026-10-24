// Reload when a newer build is published. `make build` writes version.txt (a
// hash of the bundle's index.html, which names every hashed asset); poll it
// with a throwaway query string so no cache can answer, and reload on change.
(function () {
  var INTERVAL = 60 * 1000;
  var known = null;
  var swapping = false;

  // A reload drops fullscreen and silences the sound until the next click, so
  // a viewer who has either gets a prompt instead of a surprise. Everyone else
  // is updated straight away.
  function updated() {
    var fs = !!document.fullscreenElement;
    var sound = document.getElementById('listen')?.classList.contains('gone');
    if (!fs && !sound) return reload(false);
    if (document.getElementById('update-banner')) return;
    var b = document.createElement('button');
    b.id = 'update-banner';
    b.textContent = 'A new version is ready \u2014 click to reload';
    b.addEventListener('click', function () { reload(fs); });
    document.body.appendChild(b);
  }

  function reload(fs) {
    try { if (fs) sessionStorage.setItem('meeple-fs', '1'); } catch (e) {}
    location.reload();
  }

  // Browsers only allow fullscreen from a gesture, so after a reload put it
  // back on the first click or key.
  try {
    if (sessionStorage.getItem('meeple-fs')) {
      sessionStorage.removeItem('meeple-fs');
      var restore = function () {
        window.removeEventListener('pointerup', restore);
        window.removeEventListener('keydown', restore);
        if (!document.fullscreenElement) document.documentElement.requestFullscreen?.().catch(function () {});
      };
      window.addEventListener('pointerup', restore);
      window.addEventListener('keydown', restore);
    }
  } catch (e) {}

  // Run the new wasm in this page: stop the old instance (it notices
  // window.__meepleGen has moved on), give the new one a fresh canvas and start
  // it. The world starts over, as it would after a reload.
  async function swap() {
    var html = await (await fetch('index.html?r=' + Date.now(), { cache: 'no-store' })).text();
    var js = /from '([^']*meeple-[0-9a-f]+\.js)'/.exec(html);
    var wasm = /module_or_path: '([^']*\.wasm)'/.exec(html);
    if (!js || !wasm) throw new Error('bootstrap not found');
    // the glue is often byte-identical between builds, so the query makes the
    // browser evaluate it afresh instead of handing back the running module
    var mod = await import(js[1] + '?v=' + Date.now());
    // fetch the wasm before touching the page, so a failure leaves it running
    var bytes = await (await fetch(wasm[1])).arrayBuffer();
    window.__meepleGen = (window.__meepleGen || 0) + 1;
    var old = document.getElementById('meeple');
    var fresh = old.cloneNode(false);
    try { old.getContext('webgl2').getExtension('WEBGL_lose_context').loseContext(); } catch (e) {}
    old.replaceWith(fresh);
    await mod.default({ module_or_path: bytes });
    window.wasmBindings = mod;
  }

  function changed(all, shell) {
    if (shell !== known.shell) return updated();
    if (swapping) return;
    swapping = true;
    swap().then(function () { known.all = all; }, function (e) {
      console.warn('in-place update failed', e);
      updated();
    }).then(function () { swapping = false; });
  }

  function check() {
    fetch('version.txt?r=' + Date.now() + Math.random().toString(36).slice(2), { cache: 'no-store' })
      .then(function (r) { return r.ok ? r.text() : null; })
      .then(function (t) {
        // Absent under `trunk serve`, or while offline: nothing to compare.
        var m = t && /^([0-9a-f]+) ([0-9a-f]+)\s*$/.exec(t);
        if (!m) return;
        if (known === null) known = { all: m[1], shell: m[2] };
        else if (m[1] !== known.all) changed(m[1], m[2]);
      })
      .catch(function () {});
  }

  check();
  setInterval(check, INTERVAL);
  document.addEventListener('visibilitychange', function () {
    if (!document.hidden) check();
  });
})();
