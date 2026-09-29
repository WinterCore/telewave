import logo from "../assets/artwork/channel-logo.svg";

/*
 * One configuration object supplies the branding for both channel pages.
 * Colors/fonts become `--brand-*` custom properties in App.svelte; Tailwind
 * utilities (bg-base, text-ink, font-display, …) pick them up via app.css.
 */
export const CHANNEL = {
  name: "غرفة الإنصات",
  handle: "@thelisteningroom",
  description: "حوارات واكتشافات هادئة. مساحة صغيرة للإنصات.",
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
    headingFont: "'IBM Plex Sans Arabic', Tahoma, sans-serif",
    bodyFont: "'IBM Plex Sans Arabic', Tahoma, sans-serif",
    monoFont: "'IBM Plex Mono', ui-monospace, monospace",
  },
};

export const TYPES = [
  { value: "all", label: "كل الأنواع" },
  { value: "Conversation", label: "حوارات" },
  { value: "Story", label: "حكايات" },
  { value: "Reflection", label: "تأملات" },
  { value: "Ambient", label: "أصوات محيطة" },
];

/** Keep filter values stable while rendering Arabic category names. */
export function typeLabel(value) {
  return TYPES.find((type) => type.value === value)?.label ?? value;
}

export const SORTS = [
  { value: "newest", label: "الأحدث أولًا" },
  { value: "oldest", label: "الأقدم أولًا" },
  { value: "shortest", label: "الأقصر أولًا" },
  { value: "title", label: "العنوان أبجديًا" },
];

export const RECORDINGS = [
  {
    id: "slowing-down",
    title: "فن التمهّل",
    author: "أوليفر ريد",
    type: "Conversation",
    duration: 2312,
    date: "2026-09-07",
    image: "slow-mornings",
    description: "كيف نمنح أنفسنا فسحة أكبر في تفاصيل الحياة اليومية.",
  },
  {
    id: "ordinary-things",
    title: "بهجة الأشياء البسيطة",
    author: "آنا ويلز",
    type: "Conversation",
    duration: 1674,
    date: "2026-09-06",
    image: "slow-mornings",
    description: "عن الجمال الكامن في التفاصيل التي نمرّ بها دون أن نلاحظها.",
  },
  {
    id: "making-things",
    title: "عن صنع أشياء تستحق",
    author: "سام إليس",
    type: "Reflection",
    duration: 2740,
    date: "2026-09-05",
    image: "creative",
    description: "تأملات في الإبداع وكيف نخصّص له وقتًا في حياتنا.",
  },
  {
    id: "quiet-waves",
    title: "الهدوء بين الأمواج",
    author: "تسجيل ميداني",
    type: "Ambient",
    duration: 1800,
    date: "2026-09-04",
    image: "tides",
    description: "نصف ساعة على ضفاف الماء. لا أكثر ولا أقل.",
  },
  {
    id: "really-listen",
    title: "ما معنى أن ننصت حقًا",
    author: "سام إليس",
    type: "Conversation",
    duration: 3031,
    date: "2026-09-03",
    image: "slow-mornings",
    description:
      "حوار عن الانتباه والتفهّم والحضور في اللحظة.",
  },
  {
    id: "somewhere-new",
    title: "مكان لم تزره من قبل",
    author: "مايا بروكس",
    type: "Story",
    duration: 3165,
    date: "2026-09-02",
    image: "field-notes",
    description: "حكايات من الطريق وعن أناس نلتقيهم في رحلاتنا.",
  },
  {
    id: "long-way",
    title: "العودة إلى البيت من الطريق الأطول",
    author: "مايا بروكس",
    type: "Story",
    duration: 2518,
    date: "2026-09-01",
    image: "field-notes",
    description: "منعطف غير متوقّع يفتح لنا زاوية جديدة لرؤية الأشياء.",
  },
  {
    id: "begin-again",
    title: "فرصة لبداية جديدة",
    author: "آنا ويلز",
    type: "Reflection",
    duration: 1944,
    date: "2026-08-30",
    image: "creative",
    description: "نفسح في حياتنا مكانًا لبداية جديدة.",
  },
];
