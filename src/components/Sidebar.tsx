import { SparkIcon } from "./Icon";
import type { ThemeCategory } from "../types/theme";

const CATEGORIES: ThemeCategory[] = ["Suite", "GTK", "Qt", "Icons", "Cursors", "Wallpaper"];

const nav = [
  { label: "Browse", icon: "✦", active: true },
  { label: "Installed", icon: "✓", active: false },
  { label: "Favorites", icon: "♡", active: false },
];

interface Props {
  category: ThemeCategory | "All";
  counts: Record<string, number>;
  onSelectCategory: (c: ThemeCategory | "All") => void;
}

export function Sidebar({ category, counts, onSelectCategory }: Props) {
  const cats: (ThemeCategory | "All")[] = ["All", ...CATEGORIES];
  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brand-mark">
          <SparkIcon />
        </div>
        <div className="brand-name">
          Linux<span>Themer</span>
        </div>
      </div>

      {nav.map((n) => (
        <div key={n.label} className={`nav-item ${n.active ? "active" : ""}`}>
          <span>{n.icon}</span>
          {n.label}
        </div>
      ))}

      <div className="nav-label">Categories</div>
      {cats.map((c) => (
        <div
          key={c}
          className={`nav-item ${category === c ? "active" : ""}`}
          onClick={() => onSelectCategory(c)}
        >
          {c}
          <span className="count">{counts[c] ?? 0}</span>
        </div>
      ))}

      <div className="sidebar-footer">
        <span>v0.1.0</span>
        <span className="version-chip">rust core</span>
      </div>
    </aside>
  );
}
