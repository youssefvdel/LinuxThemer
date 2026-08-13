export type SortId = "popular" | "rating" | "name";

export type View = "browse" | "installed" | "favorites" | "detail";

export type ApplyComponentId =
  | "gtk"
  | "qt"
  | "icons"
  | "cursors"
  | "wallpaper"
  | "accent";

export interface ApplyComponent {
  id: ApplyComponentId;
  label: string;
}

export const APPLY_COMPONENTS: ApplyComponent[] = [
  { id: "gtk", label: "GTK 3 / 4 theme" },
  { id: "qt", label: "Qt / Kvantum" },
  { id: "icons", label: "Icon theme" },
  { id: "cursors", label: "Cursor theme" },
  { id: "wallpaper", label: "Wallpaper" },
  { id: "accent", label: "Accent color" },
];

export interface Theme {
  id: string;
  name: string;
  author: string;
  category: string;
  tags: string[];
  downloads: number;
  rating: number;
  description: string;
  /** Screenshot / preview image URL (from OCS). */
  preview?: string;
  /** All preview images (hover gallery + detail page). */
  images?: string[];
  /** Archive download URL (from OCS). */
  downloadUrl?: string;
  /** Curated gradient fallback (only for bundled seed themes). */
  wallpaper?: [string, string, string];
  palette?: string[];
  accent?: string;
}
