/** 2312 -> "38:32"; 3725 -> "1:02:05" */
export function formatTime(seconds) {
  const total = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const secs = total % 60;
  const mm = hours > 0 ? String(minutes).padStart(2, "0") : minutes;
  return `${hours > 0 ? `${hours}:` : ""}${mm}:${String(secs).padStart(2, "0")}`;
}

/** "2026-09-07" -> "Sep 7, 2026" */
export function formatDate(iso) {
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(`${iso}T00:00:00`));
}

/** 2740 -> "about 46 min" */
export function formatApprox(seconds) {
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `about ${minutes} min`;
  return `about ${Math.floor(minutes / 60)} hr ${minutes % 60} min`;
}
