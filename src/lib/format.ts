import type { Theme } from "../types/theme";

/** "48200" -> "48.2k", "152300" -> "152.3k", "900" -> "900" */
export function fmtDownloads(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1).replace(/\.0$/, "")}k`;
  return String(n);
}

/** Builds the accent-glow wallpaper preview for a curated theme. */
export function wallpaperBackground(theme: Theme): string {
  const [a, b, c] = theme.wallpaper ?? ["#1e1e2e", "#313244", "#45475a"];
  const ac = theme.accent ?? "#8b5cf6";
  return `radial-gradient(110% 75% at 18% 0%, ${ac}40 0%, transparent 52%), radial-gradient(80% 80% at 92% 108%, ${ac}2e 0%, transparent 55%), linear-gradient(135deg, ${a} 0%, ${b} 52%, ${c} 100%)`;
}
