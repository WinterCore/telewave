// Standalone design preview. Playback is simulated; no audio or backend is connected.
(() => {
  "use strict";
  const $ = (selector) => document.querySelector(selector);
  const $$ = (selector) => [...document.querySelectorAll(selector)];
  const channel = window.TELEWAVE_CHANNEL;
  const isLive = document.body.dataset.page === "live";
  const time = (seconds) =>
    `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;
  const icon = (playing) =>
    `<svg class="size-4" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">${playing ? '<path d="M6 5h4v14H6zM14 5h4v14h-4z"/>' : '<path d="m8 5 11 7-11 7Z"/>'}</svg>`;

  const recordings = [
    {
      id: "slowing-down",
      title: "The art of slowing down",
      author: "Oliver Reed",
      type: "Conversation",
      duration: 2312,
      date: "2026-09-07",
      image: "slow-mornings",
      description: "On finding a little more space in everyday life.",
    },
    {
      id: "ordinary-things",
      title: "The joy of ordinary things",
      author: "Anna Wells",
      type: "Conversation",
      duration: 1674,
      date: "2026-09-06",
      image: "slow-mornings",
      description: "Finding something good in the things we often overlook.",
    },
    {
      id: "making-things",
      title: "On making things that matter",
      author: "Sam Ellis",
      type: "Reflection",
      duration: 2740,
      date: "2026-09-05",
      image: "creative",
      description: "A few thoughts on creativity and making time for it.",
    },
    {
      id: "quiet-waves",
      title: "The quiet between the waves",
      author: "A field recording",
      type: "Ambient",
      duration: 1800,
      date: "2026-09-04",
      image: "tides",
      description: "Half an hour by the water. Nothing more, nothing less.",
    },
    {
      id: "really-listen",
      title: "What it means to really listen",
      author: "Sam Ellis",
      type: "Conversation",
      duration: 3031,
      date: "2026-09-03",
      image: "slow-mornings",
      description:
        "A conversation about attention, understanding, and being present.",
    },
    {
      id: "somewhere-new",
      title: "Somewhere you’ve never been",
      author: "Maya Brooks",
      type: "Story",
      duration: 3165,
      date: "2026-09-02",
      image: "field-notes",
      description:
        "Stories from the road and the people we meet along the way.",
    },
    {
      id: "long-way",
      title: "Taking the long way home",
      author: "Maya Brooks",
      type: "Story",
      duration: 2518,
      date: "2026-09-01",
      image: "field-notes",
      description:
        "An unexpected detour, and a different way of seeing things.",
    },
    {
      id: "begin-again",
      title: "Permission to begin again",
      author: "Anna Wells",
      type: "Reflection",
      duration: 1944,
      date: "2026-08-30",
      image: "creative",
      description: "Making a little room for a new beginning.",
    },
  ];

  // One configuration supplies the branding for both channel pages.
  $("[data-channel-name]").textContent = channel.name;
  $("[data-channel-handle]").textContent = channel.handle;
  $("[data-channel-description]").textContent = channel.description;
  $("[data-channel-logo]").src = channel.logo;
  $("[data-channel-logo]").alt = `${channel.name} logo`;
  $('link[rel="icon"]').href = channel.logo;
  document.title = `${isLive ? "Live stream" : "Recordings"} — ${channel.name}`;
  $('meta[name="description"]').content = channel.description;
  $('meta[name="theme-color"]').content = channel.theme.background;
  const themeProperties = {
    accent: "--color-accent",
    background: "--color-paper",
    text: "--color-ink",
    muted: "--color-muted",
    border: "--color-line",
    headingFont: "--font-display",
  };
  Object.entries(themeProperties).forEach(([key, property]) =>
    document.documentElement.style.setProperty(property, channel.theme[key]),
  );

  let current = recordings[0];
  let playing = false;
  let position = 0;

  function updateProgress() {
    const progress = $(isLive ? "#live-progress" : "#seek");
    progress.max = current.duration;
    progress.value = position;
    progress.setAttribute(
      "aria-valuetext",
      `${time(position)} of ${time(current.duration)}`,
    );
    if (!isLive)
      progress.style.setProperty(
        "--progress",
        `${(position / current.duration) * 100}%`,
      );
    $(isLive ? "#live-elapsed" : "#elapsed").textContent = time(position);
    $(isLive ? "#live-duration" : "#duration").textContent = time(
      current.duration,
    );
  }

  function updatePlayer() {
    if (isLive) {
      $("#live-art").src = `assets/${current.image}.svg`;
      $("#live-art").alt = `Artwork for ${current.title}`;
      $("#live-title").textContent = current.title;
      $("#live-metadata").textContent =
        `${current.author} · ${current.type} · ${Math.floor(current.duration / 60)} min`;
      $("#live-description").textContent = current.description;
      $("#live-toggle").innerHTML =
        `${icon(playing)}${playing ? "Pause" : "Listen live"}`;
      $("#live-toggle").setAttribute(
        "aria-label",
        playing ? "Pause live stream" : "Listen live",
      );
    } else {
      $("#player-art").src = `assets/${current.image}.svg`;
      $("#player-title").textContent = current.title;
      $("#player-author").textContent = current.author;
      $("#play-toggle").innerHTML = icon(playing);
      $("#play-toggle").setAttribute(
        "aria-label",
        `${playing ? "Pause" : "Play"} ${current.title}`,
      );
      $$(".recording-row").forEach((row) => {
        const selected = row.dataset.id === current.id;
        const track = recordings.find((item) => item.id === row.dataset.id);
        row.classList.toggle("selected", selected);
        const button = row.querySelector(".row-play");
        button.innerHTML = icon(selected && playing);
        button.setAttribute(
          "aria-label",
          `${selected && playing ? "Pause" : "Play"} ${track.title}`,
        );
      });
    }
    updateProgress();
  }

  function renderRecordings() {
    const query = $("#search").value.trim().toLocaleLowerCase();
    const type = $("#filter").value;
    const sort = $("#sort").value;
    const visible = recordings.filter(
      (track) =>
        (type === "all" || track.type === type) &&
        `${track.title} ${track.author} ${track.description}`
          .toLocaleLowerCase()
          .includes(query),
    );
    visible.sort((a, b) => {
      if (sort === "title") return a.title.localeCompare(b.title);
      if (sort === "duration") return a.duration - b.duration;
      return sort === "oldest"
        ? a.date.localeCompare(b.date)
        : b.date.localeCompare(a.date);
    });
    $("#recording-count").textContent =
      `${visible.length} ${visible.length === 1 ? "recording" : "recordings"}`;
    $("#empty-state").hidden = visible.length > 0;
    $("#recordings").innerHTML = visible
      .map(
        (track) => `
      <div class="recording-grid recording-row" data-id="${track.id}" role="listitem">
        <button class="row-play" data-play="${track.id}" aria-label="Play ${track.title}">${icon(false)}</button>
        <div class="flex min-w-0 items-center gap-3 sm:gap-4">
          <img src="assets/${track.image}.svg" alt="" width="48" height="48" class="size-10 shrink-0 rounded-md object-cover sm:size-12">
          <div class="min-w-0">
            <button data-play="${track.id}" class="block max-w-full truncate text-left text-[13px] font-medium hover:text-accent sm:text-[14px]" title="${track.title}">${track.title}</button>
            <p class="mt-1 truncate text-[11px] text-muted sm:text-xs">${track.author} <span class="px-1" aria-hidden="true">·</span> ${track.type}</p>
          </div>
        </div>
        <time datetime="${track.date}" class="hidden text-xs text-muted sm:block">${new Date(`${track.date}T12:00:00`).toLocaleDateString("en-US", { month: "short", day: "numeric" })}</time>
        <span class="text-right text-[11px] text-muted tabular-nums sm:text-xs">${time(track.duration)}</span>
      </div>`,
      )
      .join("");
    updatePlayer();
  }

  function togglePlayback() {
    if (!isLive && position >= current.duration) position = 0;
    playing = !playing;
    updatePlayer();
  }

  // The live preview loops channel recordings, displaying only the current file.
  const liveStartedAt = Date.now() - 742000;
  function syncLive() {
    const duration = recordings.reduce((sum, track) => sum + track.duration, 0);
    let elapsed = Math.floor((Date.now() - liveStartedAt) / 1000) % duration;
    let index = 0;
    while (elapsed >= recordings[index].duration)
      elapsed -= recordings[index++].duration;
    const changed = current.id !== recordings[index].id;
    current = recordings[index];
    position = elapsed;
    if (changed) updatePlayer();
    else updateProgress();
  }

  if (isLive) {
    $("#live-toggle").addEventListener("click", togglePlayback);
    syncLive();
    updatePlayer();
  } else {
    $("#search").addEventListener("input", renderRecordings);
    $("#filter").addEventListener("change", renderRecordings);
    $("#sort").addEventListener("change", renderRecordings);
    $("#clear-filters").addEventListener("click", () => {
      $("#search").value = "";
      $("#filter").value = "all";
      renderRecordings();
    });
    $("#recordings").addEventListener("click", (event) => {
      const button = event.target.closest("[data-play]");
      if (!button) return;
      const track = recordings.find((item) => item.id === button.dataset.play);
      if (track.id === current.id) togglePlayback();
      else {
        current = track;
        position = 0;
        playing = true;
        updatePlayer();
      }
    });
    $("#play-toggle").addEventListener("click", togglePlayback);
    $("#seek").addEventListener("input", (event) => {
      position = Number(event.target.value);
      updateProgress();
    });
    renderRecordings();
  }

  let lastTick = performance.now();
  setInterval(() => {
    const now = performance.now();
    const elapsed = (now - lastTick) / 1000;
    lastTick = now;
    if (isLive) syncLive();
    else if (playing) {
      position = Math.min(current.duration, position + elapsed);
      if (position === current.duration) {
        playing = false;
        updatePlayer();
      } else updateProgress();
    }
  }, 1000);
})();
