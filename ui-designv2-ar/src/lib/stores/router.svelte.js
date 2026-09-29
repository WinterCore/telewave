/** Tiny hash router: "#/" -> recordings, "#/live" -> live stream. */
const parse = () => (location.hash === "#/live" ? "live" : "recordings");

export const router = $state({ page: parse() });

window.addEventListener("hashchange", () => {
  router.page = parse();
});
