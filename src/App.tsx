import { useEffect, useMemo, useState } from "react";
import { loadThemes } from "./lib/store";
import {
  APPLY_COMPONENTS,
  type SortId,
  type Theme,
  type ThemeCategory,
} from "./types/theme";
import { Sidebar } from "./components/Sidebar";
import { Topbar } from "./components/Topbar";
import { Hero } from "./components/Hero";
import { ThemeGrid } from "./components/ThemeGrid";
import { ApplyModal } from "./components/ApplyModal";

export default function App() {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<ThemeCategory | "All">("All");
  const [sort, setSort] = useState<SortId>("popular");
  const [installed, setInstalled] = useState<Set<string>>(
    new Set(["catppuccin-mocha"])
  );
  const [applying, setApplying] = useState<Theme | null>(null);
  const [components, setComponents] = useState<Set<string>>(
    new Set(APPLY_COMPONENTS.map((c) => c.id))
  );
  const [themes, setThemes] = useState<Theme[]>([]);

  useEffect(() => {
    loadThemes().then(setThemes);
  }, []);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    const list = themes.filter((t) => {
      const matchCat = category === "All" || t.category === category;
      const matchQ =
        !q ||
        t.name.toLowerCase().includes(q) ||
        t.author.toLowerCase().includes(q) ||
        t.tags.some((tag) => tag.includes(q));
      return matchCat && matchQ;
    });
    return [...list].sort((a, b) => {
      if (sort === "popular") return b.downloads - a.downloads;
      if (sort === "rating") return b.rating - a.rating;
      return a.name.localeCompare(b.name);
    });
  }, [query, category, sort, themes]);

  const counts = useMemo(() => {
    const c: Record<string, number> = { All: themes.length };
    for (const t of themes) c[t.category] = (c[t.category] ?? 0) + 1;
    return c;
  }, [themes]);

  const toggleInstall = (id: string) => {
    setInstalled((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const openApply = (t: Theme) => {
    setComponents(new Set(APPLY_COMPONENTS.map((c) => c.id)));
    setApplying(t);
  };

  const toggleComponent = (id: string) => {
    setComponents((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const confirmApply = () => {
    if (applying) toggleInstall(applying.id);
    setApplying(null);
  };

  return (
    <div className="app">
      <Sidebar
        category={category}
        counts={counts}
        onSelectCategory={setCategory}
      />
      <div className="main">
        <Topbar
          count={filtered.length}
          sort={sort}
          onSort={setSort}
          query={query}
          onQuery={setQuery}
        />
        <div className="content">
          {themes[0] && (
            <Hero
              theme={themes[0]}
              installed={installed.has(themes[0].id)}
              onApply={openApply}
            />
          )}
          <div className="section-head">
            <h3>All themes</h3>
            <span className="hint">
              {category === "All" ? "Every unified manifest in the store" : category}
            </span>
          </div>
          <ThemeGrid themes={filtered} installed={installed} onApply={openApply} />
          {filtered.length === 0 && themes.length > 0 && (
            <div className="empty">No themes match “{query}”.</div>
          )}
        </div>
      </div>
      {applying && (
        <ApplyModal
          theme={applying}
          selected={components}
          onToggle={toggleComponent}
          onCancel={() => setApplying(null)}
          onConfirm={confirmApply}
        />
      )}
    </div>
  );
}
