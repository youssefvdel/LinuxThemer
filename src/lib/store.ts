import { invoke } from "@tauri-apps/api/core";
import { themes as seedThemes } from "../data/themes";
import type { Theme } from "../types/theme";

export interface StoreCategory {
  id: string;
  label: string;
}

/** Category IDs from the KDE-Look / opendesktop OCS API. */
export const STORE_CATEGORIES: StoreCategory[] = [
  { id: "722", label: "Global Themes" },
  { id: "132", label: "Icons" },
  { id: "107", label: "Cursors" },
  { id: "135", label: "GTK Themes" },
  { id: "134", label: "GNOME Shell" },
  { id: "112", label: "Color Schemes" },
  { id: "104", label: "Plasma Themes" },
  { id: "114", label: "Decorations" },
  { id: "101", label: "SDDM" },
  { id: "295", label: "Wallpapers" },
];

interface OcsItem {
  id: number;
  name: string;
  personid?: string;
  typename?: string;
  downloads?: string;
  score?: number;
  summary?: string;
  description?: string;
  previewpic1?: string;
  smallpreviewpic1?: string;
  downloadlink1?: string;
  tags?: string;
}

interface OcsResponse {
  data?: OcsItem[];
  totalitems?: number;
}

function stripHtml(s: string): string {
  return s
    .replace(/<[^>]*>/g, "")
    .replace(/\\u003C[^>]*\\u003E/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function mapOcs(i: OcsItem): Theme {
  return {
    id: String(i.id),
    name: i.name,
    author: i.personid ?? "unknown",
    category: i.typename ?? "",
    tags: (i.tags ?? "")
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean),
    downloads: parseInt(i.downloads ?? "0", 10) || 0,
    rating: Math.round(((i.score ?? 50) / 20) * 10) / 10,
    description: stripHtml(i.summary || i.description || ""),
    preview: i.previewpic1 ?? i.smallpreviewpic1,
    downloadUrl: i.downloadlink1,
  };
}

const hasTauri =
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

export async function fetchStore(
  category: string,
  page: number,
  search: string
): Promise<{ themes: Theme[]; total: number }> {
  if (!hasTauri) {
    const q = search.trim().toLowerCase();
    const filtered = seedThemes.filter(
      (t) => !q || t.name.toLowerCase().includes(q)
    );
    return { themes: filtered, total: filtered.length };
  }

  const raw = await invoke<OcsResponse>("fetch_themes", { category, page, search });
  return {
    themes: (raw.data ?? []).map(mapOcs),
    total: raw.totalitems ?? 0,
  };
}
