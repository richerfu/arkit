import "@fontsource/maple-mono/index.css";
import "./styles.css";

if (import.meta.env.DEV) {
  void import("./dev");
}

document.addEventListener("click", (event) => {
  if (!(event.target instanceof Element)) return;
  const button = event.target.closest<HTMLButtonElement>("button.copy");
  const code = button?.parentElement?.querySelector("pre code")?.textContent;
  if (!button || !code || !navigator.clipboard) return;

  void navigator.clipboard.writeText(code).then(() => {
    const title = button.title;
    button.title = "Copied";
    button.dataset.copied = "true";
    window.setTimeout(() => {
      button.title = title;
      delete button.dataset.copied;
    }, 1_200);
  });
});
