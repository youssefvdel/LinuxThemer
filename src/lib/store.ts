import { themes as fallbackThemes } from "../data/themes";
import type { Theme } from "../types/theme";

// Store registry URL. Override with VITE_STORE_URL (e.g. a GitHub raw URL)
// once the curated registry is hosted. Defaults to the bundled JSON served
// by this app itself at /store/index.json.
const STORE_URL: string =
  (import.meta.env.VITE_STORE_URL as string | undefined) ?? "/store/index.json";

/** Fetches the theme registry over HTTP; falls back to bundled seed offline. */
export async function loadThemes(): Promise<Theme[]> {
  try {
    const res = await fetch(STORE_URL, { cache: "no-cache" });
    if (!res.ok) throw new Error(`store HTTP ${res.status}`);
    const data = (await res.json()) as Theme[];
    if (Array.isArray(data) && data.length > 0) return data;
    throw new Error("store returned no themes");
  } catch {
    return fallbackThemes;
  }
}
