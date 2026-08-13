import { useEffect, useMemo, useState } from "react";
import { fetchStore, STORE_CATEGORIES } from "./lib/store";
import { APPLY_COMPONENTS, type SortId, type Theme } from "./types/theme";
import { Sidebar } from "./components/Sidebar";
import { Topbar } from "./components/Topbar";
import { Hero } from "./components/Hero";
import { ThemeGrid } from "./components/ThemeGrid";
import { ApplyModal } from "./components/ApplyModal";

export default function App() {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("135");
  const [sort, setSort] = useState<SortId>("popular");
  const [themes, setThemes] = useState<Theme[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(false);
  const [installed, setInstalled] = useState<Set<string>>(new Set());
  const [applying, setApplying] = useState<Theme | null>(null);
  const [components, setComponents] = useState<Set<string>>(
    new Set(APPLY_COMPONENTS.map((c) => c.id))
  );

  useEffect(() => {
    const t = setTimeout(() => {
      let cancelled = false;
      setLoading(true);
      fetchStore(category, 0, query)
        .then(({ themes: list, total: n }) => {
          if (cancelled) return;
          setThemes(list);
          setTotal(n);
          setPage(0);
          setLoading(false);
        })
        .catch(() => {
          if (!cancelled) setLoading(false);
        });
      return () => {
        cancelled = true;
      };
    }, 300);
    return () => clearTimeout(t);
  }, [category, query]);

  const loadMore = async () => {
    setLoading(true);
    try {
      const next = page + 1;
      const { themes: more } = await fetchStore(category, next, query);
      setThemes((prev) => [...prev, ...more]);
      setPage(next);
    } finally {
      setLoading(false);
    }
  };

  const filtered = useMemo(() => {
    return [...themes].sort((a, b) => {
      if (sort === "popular") return b.downloads - a.downloads;
      if (sort === "rating") return b.rating - a.rating;
      return a.name.localeCompare(b.name);
    });
  }, [themes, sort]);

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

  const categoryLabel =
    STORE_CATEGORIES.find((c) => c.id === category)?.label ?? "Themes";
  const hasMore = themes.length < total;

  return (
    <div className="app">
      <Sidebar
        categories={STORE_CATEGORIES}
        activeCategory={category}
        onSelectCategory={setCategory}
      />
      <div className="main">
        <Topbar
          count={themes.length}
          sort={sort}
          onSort={setSort}
          query={query}
          onQuery={setQuery}
        />
        <div className="content">
          {filtered[0] && (
            <Hero
              theme={filtered[0]}
              installed={installed.has(filtered[0].id)}
              onApply={openApply}
            />
          )}
          <div className="section-head">
            <h3>{categoryLabel}</h3>
            <span className="hint">
              {total.toLocaleString()} themes · page {page + 1}
            </span>
          </div>
          <ThemeGrid themes={filtered} installed={installed} onApply={openApply} />
          {loading && <div className="empty">Loading…</div>}
          {!loading && filtered.length === 0 && (
            <div className="empty">No themes found.</div>
          )}
          {hasMore && !loading && (
            <div className="load-more">
              <button className="btn btn-ghost" onClick={loadMore}>
                Load more
              </button>
            </div>
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
