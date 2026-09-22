import { RECORDINGS } from "../channel.js";

/*
 * Simulated playback engine for the Recordings page. No audio is attached;
 * position simply advances against the wall clock while "playing".
 */
export const player = $state({
  recordingId: null,
  playing: false,
  position: 0,
});

const durationOf = (id) =>
  RECORDINGS.find((recording) => recording.id === id)?.duration ?? 0;

let timer = null;
let last = 0;

function tick() {
  const now = performance.now();
  player.position = Math.min(
    durationOf(player.recordingId),
    player.position + (now - last) / 1000,
  );
  last = now;
  if (player.position >= durationOf(player.recordingId)) {
    stopTimer();
    player.playing = false;
  }
}

function startTimer() {
  last = performance.now();
  clearInterval(timer);
  timer = setInterval(tick, 250);
}

function stopTimer() {
  clearInterval(timer);
  timer = null;
}

/** Play a recording: a new one starts from the top, the current one resumes. */
export function play(id) {
  if (id !== player.recordingId) {
    player.recordingId = id;
    player.position = 0;
  } else if (player.position >= durationOf(id)) {
    player.position = 0; // replay a finished recording
  }
  player.playing = true;
  startTimer();
}

export function pause() {
  player.playing = false;
  stopTimer();
}

/** Play/pause semantics for the row and player-bar buttons. */
export function toggle(id) {
  if (player.playing && player.recordingId === id) pause();
  else play(id);
}

export function seek(position) {
  player.position = Math.min(
    Math.max(0, position),
    durationOf(player.recordingId),
  );
}
