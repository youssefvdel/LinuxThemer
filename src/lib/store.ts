import { invoke } from "@tauri-apps/api/core";
import { themes as seedThemes } from "../data/themes";
import type { SortId, Theme } from "../types/theme";

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
  previewpic2?: string;
  previewpic3?: string;
  previewpic4?: string;
  previewpic5?: string;
  previewpic6?: string;
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
    .replace(/\\s+/g, " ")
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
    images: [
      i.previewpic1,
      i.previewpic2,
      i.previewpic3,
      i.previewpic4,
      i.previewpic5,
      i.previewpic6,
    ].filter((p): p is string => !!p),
    downloadUrl: i.downloadlink1,
  };
}

const hasTauri =
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

/** OCS API sortmode, keyed by UI sort id. Server-side sort keeps pagination correct. */
const SORT_MODE: Record<SortId, string> = {
  popular: "down",
  rating: "high",
  name: "alpha",
};

export async function fetchStore(
  category: string,
  page: number,
  search: string,
  sort: SortId
): Promise<{ themes: Theme[]; total: number }> {
  if (!hasTauri) {
    const q = search.trim().toLowerCase();
    const list = seedThemes.filter(
      (t) => !q || t.name.toLowerCase().includes(q)
    );
    if (sort === "popular") list.sort((a, b) => b.downloads - a.downloads);
    else if (sort === "rating") list.sort((a, b) => b.rating - a.rating);
    else list.sort((a, b) => a.name.localeCompare(b.name));
    return { themes: list, total: list.length };
  }

  const raw = await invoke<OcsResponse>("fetch_themes", {
    category,
    page,
    search,
    sortmode: SORT_MODE[sort],
  });
  return {
    themes: (raw.data ?? []).map(mapOcs),
    total: raw.totalitems ?? 0,
  };
}
