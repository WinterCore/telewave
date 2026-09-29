const LOCALE = "ar-u-nu-arab";
const numberFormat = new Intl.NumberFormat(LOCALE, { useGrouping: false });
const paddedNumberFormat = new Intl.NumberFormat(LOCALE, {
  useGrouping: false,
  minimumIntegerDigits: 2,
});
const dateFormat = new Intl.DateTimeFormat(LOCALE, {
  calendar: "gregory",
  month: "long",
  day: "numeric",
  year: "numeric",
});
const minuteFormat = new Intl.NumberFormat(LOCALE, {
  style: "unit",
  unit: "minute",
  unitDisplay: "long",
});
const hourFormat = new Intl.NumberFormat(LOCALE, {
  style: "unit",
  unit: "hour",
  unitDisplay: "long",
});
const listFormat = new Intl.ListFormat("ar", { type: "conjunction" });
const pluralRules = new Intl.PluralRules("ar");

export function formatNumber(value, minimumDigits = 1) {
  return (minimumDigits === 2 ? paddedNumberFormat : numberFormat).format(value);
}

/** Use an isolated LTR element for colon-separated Arabic time digits. */
export function formatTime(seconds) {
  const total = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const secs = total % 60;
  const mm = formatNumber(minutes, hours > 0 ? 2 : 1);
  return `${hours > 0 ? `${formatNumber(hours)}:` : ""}${mm}:${formatNumber(secs, 2)}`;
}

export function formatDate(iso) {
  return dateFormat.format(new Date(`${iso}T00:00:00`));
}

export function formatApprox(seconds) {
  const minutes = Math.max(0, Math.round(seconds / 60));
  if (minutes < 60) return `${minuteFormat.format(minutes)} تقريبًا`;
  const parts = [hourFormat.format(Math.floor(minutes / 60))];
  if (minutes % 60) parts.push(minuteFormat.format(minutes % 60));
  return `${listFormat.format(parts)} تقريبًا`;
}

export function formatRecordingCount(count) {
  switch (pluralRules.select(count)) {
    case "zero":
      return "لا توجد تسجيلات";
    case "one":
      return "تسجيل واحد";
    case "two":
      return "تسجيلان";
    case "few":
      return `${formatNumber(count)} تسجيلات`;
    case "many":
      return `${formatNumber(count)} تسجيلًا`;
    default:
      return `${formatNumber(count)} تسجيل`;
  }
}

/** Match common Arabic spellings without requiring diacritics or tatweel. */
export function normalizeSearch(value) {
  return value
    .normalize("NFKC")
    .replace(/[\u0610-\u061a\u064b-\u065f\u0670\u06d6-\u06ed\u0640]/gu, "")
    .replace(/[أإآٱ]/gu, "ا")
    .replace(/ى/gu, "ي")
    .trim()
    .toLocaleLowerCase("ar");
}
