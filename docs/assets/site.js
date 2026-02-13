(() => {
  const yearEl = document.querySelector("[data-year]");
  if (yearEl) yearEl.textContent = String(new Date().getFullYear());

  const tabs = Array.from(document.querySelectorAll("[data-tab]"));
  const panes = Array.from(document.querySelectorAll(".demo-pane"));
  if (tabs.length === 0 || panes.length === 0) return;

  const paneById = new Map(panes.map((p) => [p.id, p]));

  function select(tabId, { focus = false } = {}) {
    const pane = paneById.get(tabId);
    if (!pane) return;

    for (const t of tabs) {
      const isSelected = t.getAttribute("data-tab") === tabId;
      t.setAttribute("aria-selected", isSelected ? "true" : "false");
      if (focus && isSelected) t.focus();
    }

    for (const p of panes) {
      p.setAttribute("aria-hidden", p.id === tabId ? "false" : "true");
    }
  }

  for (const t of tabs) {
    t.addEventListener("click", () => {
      select(t.getAttribute("data-tab"), { focus: true });
    });

    t.addEventListener("keydown", (e) => {
      if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
      e.preventDefault();

      const idx = tabs.indexOf(t);
      const nextIdx =
        e.key === "ArrowRight"
          ? (idx + 1) % tabs.length
          : (idx - 1 + tabs.length) % tabs.length;
      const nextTab = tabs[nextIdx];
      select(nextTab.getAttribute("data-tab"), { focus: true });
    });
  }

  // Pick a sensible default based on the URL hash, otherwise keep the initial HTML.
  const hashId = window.location.hash.replace("#", "");
  if (hashId && paneById.has(hashId)) {
    select(hashId);
  }
})();

