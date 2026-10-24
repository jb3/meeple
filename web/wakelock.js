// Keep the screen awake while the world is on show (it's meant to be watched
// like a screensaver). The browser drops the lock whenever the tab is hidden,
// so take it again on return.
(function () {
  if (!('wakeLock' in navigator)) return;
  var lock = null;

  function acquire() {
    if (lock || document.hidden) return;
    navigator.wakeLock.request('screen').then(function (l) {
      lock = l;
      l.addEventListener('release', function () { lock = null; });
    }).catch(function () {});
  }

  acquire();
  document.addEventListener('visibilitychange', acquire);
  // some browsers refuse until the page has been interacted with
  ['pointerdown', 'keydown', 'touchstart'].forEach(function (ev) {
    window.addEventListener(ev, acquire, { passive: true });
  });
})();
