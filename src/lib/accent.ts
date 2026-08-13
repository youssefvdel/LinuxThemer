export function parseHex(hex: string): { r: number; g: number; b: number } {
  const h = hex.replace("#", "");
  const full =
    h.length === 3
      ? h
          .split("")
          .map((c) => c + c)
          .join("")
      : h;
  return {
    r: parseInt(full.slice(0, 2), 16) || 0,
    g: parseInt(full.slice(2, 4), 16) || 0,
    b: parseInt(full.slice(4, 6), 16) || 0,
  };
}

/** Set the app accent CSS variables (accent / soft / on-accent) from a hex. */
export function applyAccent(hex: string): void {
  const root = document.documentElement;
  const { r, g, b } = parseHex(hex);
  root.style.setProperty("--accent", hex);
  root.style.setProperty("--accent-strong", hex);
  root.style.setProperty("--accent-soft", `rgba(${r}, ${g}, ${b}, 0.18)`);
  const lum = 0.299 * r + 0.587 * g + 0.114 * b;
  root.style.setProperty("--on-accent", lum > 150 ? "#0b0b0f" : "#ffffff");
}
