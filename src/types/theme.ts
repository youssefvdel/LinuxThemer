export type SortId = "popular" | "rating" | "name";

export type View = "browse" | "installed" | "favorites" | "detail" | "studio" | "settings";

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

/** A theme discovered on this device (filesystem scan). */
export interface InstalledTheme {
  id: string;
  name: string;
  kind: string;
  path: string;
  /** Absolute path to a preview image/video on disk (may be absent). */
  preview?: string;
  /** Representative colors (color schemes / GTK css) for a mock-window. */
  palette?: string[];
  /** Sample images (icon themes: a few icons; cursor themes: rendered cursors). */
  samples?: string[];
}

/** The user's currently-applied theme (read from live config files). */
export interface CurrentTheme {
  widgetStyle: string;
  colorScheme: string;
  iconTheme: string;
  cursorTheme: string;
  gtkTheme: string;
  plasmaTheme: string;
  kvantum: string;
  accentColor: string;
}

/** Components assembled into a new global theme by the Studio. */
export interface GlobalThemeSpec {
  gtk: string;
  widgetStyle: string;
  kvantum: string;
  icons: string;
  cursors: string;
  colors: string;
  plasma: string;
}

export const INSTALLED_KIND_LABELS: Record<string, string> = {
  global: "Global Themes",
  gtk: "GTK Themes",
  plasma: "Plasma Themes",
  icons: "Icons",
  cursors: "Cursors",
  decorations: "Decorations",
  colors: "Color Schemes",
  sddm: "SDDM",
  wallpapers: "Wallpapers",
  kvantum: "Kvantum",
  custom: "Custom",
};
