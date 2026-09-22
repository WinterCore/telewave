import logo from "../assets/artwork/channel-logo.svg";

/*
 * One configuration object supplies the branding for both channel pages.
 * Colors/fonts become `--brand-*` custom properties in App.svelte; Tailwind
 * utilities (bg-base, text-ink, font-display, …) pick them up via app.css.
 */
export const CHANNEL = {
  name: "The Listening Room",
  handle: "@thelisteningroom",
  description: "Conversations and quiet discoveries. A little room to listen.",
  logo,
  theme: {
    background: "#0f131a",
    surface: "#171d26",
    ink: "#edf1f7",
    muted: "#93a0b0",
    line: "#29313d",
    accent: "#5eead4",
    accentInk: "#062823",
    live: "#ff5c47",
    headingFont: "'Space Grotesk', system-ui, sans-serif",
    bodyFont: "'Inter', system-ui, sans-serif",
    monoFont: "'IBM Plex Mono', ui-monospace, monospace",
  },
};

export const TYPES = [
  { value: "all", label: "All types" },
  { value: "Conversation", label: "Conversations" },
  { value: "Story", label: "Stories" },
  { value: "Reflection", label: "Reflections" },
  { value: "Ambient", label: "Ambient" },
];

export const SORTS = [
  { value: "newest", label: "Newest first" },
  { value: "oldest", label: "Oldest first" },
  { value: "shortest", label: "Shortest first" },
  { value: "title", label: "Title A–Z" },
];

export const RECORDINGS = [
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
    description: "Stories from the road and the people we meet along the way.",
  },
  {
    id: "long-way",
    title: "Taking the long way home",
    author: "Maya Brooks",
    type: "Story",
    duration: 2518,
    date: "2026-09-01",
    image: "field-notes",
    description: "An unexpected detour, and a different way of seeing things.",
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
