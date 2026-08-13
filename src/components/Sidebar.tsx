import { SparkIcon } from "./Icon";
import type { StoreCategory } from "../lib/store";
import type { View } from "../types/theme";

const nav = [
  { id: "browse", label: "Browse", icon: "✦" },
  { id: "installed", label: "Installed", icon: "✓" },
  { id: "favorites", label: "Favorites", icon: "♡" },
  { id: "studio", label: "Studio", icon: "✎" },
] as const;

interface Props {
  categories: StoreCategory[];
  activeCategory: string;
  onSelectCategory: (id: string) => void;
  view: View;
  onSelectView: (v: View) => void;
  installedCount: number;
  favoritesCount: number;
}

export function Sidebar({
  categories,
  activeCategory,
  onSelectCategory,
  view,
  onSelectView,
  installedCount,
  favoritesCount,
}: Props) {
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
        <div
          key={n.id}
          className={`nav-item ${view === n.id ? "active" : ""}`}
          onClick={() => onSelectView(n.id)}
        >
          <span>{n.icon}</span>
          {n.label}
          {n.id === "installed" && <span className="count">{installedCount}</span>}
          {n.id === "favorites" && <span className="count">{favoritesCount}</span>}
        </div>
      ))}

      <div className="nav-label">Categories</div>
      {categories.map((c) => (
        <div
          key={c.id}
          className={`nav-item ${view === "browse" && activeCategory === c.id ? "active" : ""}`}
          onClick={() => onSelectCategory(c.id)}
        >
          {c.label}
        </div>
      ))}

      <div className="sidebar-footer">
        <span>v0.1.0</span>
        <span className="version-chip">kde-look</span>
      </div>
    </aside>
  );
}
