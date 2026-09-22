import { RECORDINGS } from "../channel.js";

const TOTAL = RECORDINGS.reduce((sum, recording) => sum + recording.duration, 0);

// Start the mock schedule mid-programme (40% into "On making things that
// matter") so the demo shows live progress right away.
const EPOCH = Date.now() - 5082 * 1000;

export const broadcast = $state({ listening: false, now: Date.now() });

setInterval(() => {
  broadcast.now = Date.now();
}, 1000);

/**
 * What is on air right now. Derived from the wall clock, so the timeline
 * keeps moving whether or not anyone is listening — pausing and resuming
 * simply rejoins the current position.
 */
export function onAir() {
  let elapsed = (((broadcast.now - EPOCH) / 1000) % TOTAL + TOTAL) % TOTAL;
  for (const recording of RECORDINGS) {
    if (elapsed < recording.duration) return { recording, position: elapsed };
    elapsed -= recording.duration;
  }
  const [first] = RECORDINGS;
  return { recording: first, position: 0 };
}

export function toggleListen() {
  broadcast.listening = !broadcast.listening;
}
