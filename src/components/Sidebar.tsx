import { SparkIcon } from "./Icon";
import type { StoreCategory } from "../lib/store";

const nav = [
  { label: "Browse", icon: "✦", active: true },
  { label: "Installed", icon: "✓", active: false },
  { label: "Favorites", icon: "♡", active: false },
];

interface Props {
  categories: StoreCategory[];
  activeCategory: string;
  onSelectCategory: (id: string) => void;
}

export function Sidebar({ categories, activeCategory, onSelectCategory }: Props) {
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
      {categories.map((c) => (
        <div
          key={c.id}
          className={`nav-item ${activeCategory === c.id ? "active" : ""}`}
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
