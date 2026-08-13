export type ThemeCategory =
  | "Suite"
  | "GTK"
  | "Qt"
  | "Icons"
  | "Cursors"
  | "Wallpaper";

export type SortId = "popular" | "rating" | "name";

export interface Theme {
  id: string;
  name: string;
  author: string;
  category: ThemeCategory;
  tags: string[];
  downloads: number;
  rating: number;
  wallpaper: [string, string, string];
  palette: string[];
  accent: string;
  description: string;
}

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
