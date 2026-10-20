'use strict';
(() => {
  const key = 'lab5-puzzle-countdown-v1';
  const $ = id => document.getElementById(id);
  let state = {duration:7200000, remaining:7200000, end:null, started:false};
  try {
    const saved = JSON.parse(localStorage.getItem(key));
    if (saved && Number.isFinite(saved.duration) && saved.duration >= 60000 && saved.duration <= 14400000 &&
        Number.isFinite(saved.remaining) && saved.remaining >= 0 && saved.remaining <= saved.duration &&
        (saved.end === null || Number.isFinite(saved.end)) && typeof saved.started === 'boolean') state = saved;
  } catch {}
  const save = () => {try {localStorage.setItem(key, JSON.stringify(state));} catch {}};
  const left = () => state.end === null ? state.remaining : Math.max(0, Math.min(state.duration, state.end - Date.now()));
  function render() {
    const remaining = left();
    if (remaining === 0 && state.end !== null) {state.remaining=0; state.end=null; save();}
    const seconds = Math.ceil(remaining / 1000);
    $('clock').textContent = `${String(Math.floor(seconds / 60)).padStart(2,'0')}:${String(seconds % 60).padStart(2,'0')}`;
    $('progress').value = remaining / state.duration;
    $('toggle').textContent = remaining === 0 ? 'Start again' : state.end !== null ? 'Pause' : state.started ? 'Resume' : 'Start';
    const status = remaining === 0 ? 'Time is up · Save your work and prepare your explanation.' : state.end !== null ? 'Challenge in progress' : state.started ? 'Paused' : `Ready · ${state.duration / 60000} minutes`;
    if ($('status').textContent !== status) $('status').textContent = status;
    document.body.classList.toggle('urgent', remaining > 0 && remaining <= 300000);
    document.body.classList.toggle('finished', remaining === 0);
    document.title = `${$('clock').textContent} · Lab 5 challenge`;
  }
  function reset() {state.remaining=state.duration; state.end=null; state.started=false; save(); render();}
  function duration(minutes) {
    if (!Number.isInteger(minutes) || minutes < 1 || minutes > 240) {
      $('status').textContent='Enter a whole number from 1 to 240 minutes.'; return;
    }
    if (state.end !== null) {$('status').textContent='Pause the countdown before changing its duration.'; return;}
    state.duration=minutes*60000; $('minutes').value=minutes; reset();
  }
  $('toggle').addEventListener('click', () => {
    if (state.end !== null) {state.remaining=left(); state.end=null;}
    else {if (left() === 0) state.remaining=state.duration; state.end=Date.now()+state.remaining; state.started=true;}
    save(); render();
  });
  $('reset').addEventListener('click', reset);
  $('set').addEventListener('click', () => duration(Number($('minutes').value)));
  document.querySelectorAll('[data-minutes]').forEach(button => button.addEventListener('click', () => duration(Number(button.dataset.minutes))));
  $('fullscreen').addEventListener('click', async () => {
    try {
      if (document.fullscreenElement) await document.exitFullscreen();
      else if (document.documentElement.requestFullscreen) await document.documentElement.requestFullscreen();
      else $('status').textContent='Use the browser fullscreen control (Control + Command + F on Mac).';
    } catch {$('status').textContent='Use the browser fullscreen control (Control + Command + F on Mac).';}
  });
  $('minutes').value=state.duration/60000;
  document.addEventListener('visibilitychange', render);
  setInterval(render, 250);
  render();
})();
