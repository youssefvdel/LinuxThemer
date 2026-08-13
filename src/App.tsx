import { useEffect, useRef, useState } from "react";
import { fetchStore, STORE_CATEGORIES } from "./lib/store";
import { APPLY_COMPONENTS, type SortId, type Theme, type View } from "./types/theme";
import { Sidebar } from "./components/Sidebar";
import { Topbar } from "./components/Topbar";
import { ThemeGrid } from "./components/ThemeGrid";
import { ApplyModal } from "./components/ApplyModal";
import { ThemeDetail } from "./components/ThemeDetail";

function loadThemes(key: string): Map<string, Theme> {
  try {
    const arr = JSON.parse(localStorage.getItem(key) ?? "[]") as Theme[];
    return new Map(arr.map((t) => [t.id, t]));
  } catch {
    return new Map();
  }
}

export default function App() {
  const [view, setView] = useState<View>("browse");
  const [prevView, setPrevView] = useState<View>("browse");
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("135");
  const [sort, setSort] = useState<SortId>("popular");
  const [themes, setThemes] = useState<Theme[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(false);
  const [installed, setInstalled] = useState<Map<string, Theme>>(() =>
    loadThemes("lt.installed")
  );
  const [favorites, setFavorites] = useState<Map<string, Theme>>(() =>
    loadThemes("lt.favorites")
  );
  const [selected, setSelected] = useState<Theme | null>(null);
  const [applying, setApplying] = useState<Theme | null>(null);
  const [components, setComponents] = useState<Set<string>>(
    new Set(APPLY_COMPONENTS.map((c) => c.id))
  );

  const sentinelRef = useRef<HTMLDivElement | null>(null);

  // Persist installed + favorites (full theme objects, not just ids).
  useEffect(() => {
    localStorage.setItem("lt.installed", JSON.stringify([...installed.values()]));
  }, [installed]);
  useEffect(() => {
    localStorage.setItem("lt.favorites", JSON.stringify([...favorites.values()]));
  }, [favorites]);

  // Fetch first page whenever category / search / sort changes (browse only).
  useEffect(() => {
    if (view !== "browse") return;
    const t = setTimeout(() => {
      let cancelled = false;
      setLoading(true);
      fetchStore(category, 0, query, sort)
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
  }, [category, query, sort, view]);

  const loadMore = async () => {
    if (loading || themes.length >= total) return;
    setLoading(true);
    try {
      const next = page + 1;
      const { themes: more } = await fetchStore(category, next, query, sort);
      setThemes((prev) => [...prev, ...more]);
      setPage(next);
    } finally {
      setLoading(false);
    }
  };

  const loadMoreRef = useRef(loadMore);
  useEffect(() => {
    loadMoreRef.current = loadMore;
  });

  // Infinite scroll (browse only).
  useEffect(() => {
    if (view !== "browse") return;
    const el = sentinelRef.current;
    if (!el) return;
    const obs = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting) loadMoreRef.current();
      },
      { rootMargin: "600px" }
    );
    obs.observe(el);
    return () => obs.disconnect();
  }, [view]);

  const toggleInstall = (t: Theme) => {
    setInstalled((prev) => {
      const next = new Map(prev);
      if (next.has(t.id)) next.delete(t.id);
      else next.set(t.id, t);
      return next;
    });
  };

  const toggleFavorite = (t: Theme) => {
    setFavorites((prev) => {
      const next = new Map(prev);
      if (next.has(t.id)) next.delete(t.id);
      else next.set(t.id, t);
      return next;
    });
  };

  const openDetail = (t: Theme) => {
    setPrevView(view);
    setSelected(t);
    setView("detail");
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
    if (applying) toggleInstall(applying);
    setApplying(null);
  };

  const selectCategory = (id: string) => {
    setCategory(id);
    setView("browse");
  };

  const categoryLabel =
    STORE_CATEGORIES.find((c) => c.id === category)?.label ?? "Themes";

  return (
    <div className="app">
      <Sidebar
        categories={STORE_CATEGORIES}
        activeCategory={category}
        onSelectCategory={selectCategory}
        view={view}
        onSelectView={setView}
        installedCount={installed.size}
        favoritesCount={favorites.size}
      />
      <div className="main">
        {view === "browse" && (
          <>
            <Topbar
              count={themes.length}
              sort={sort}
              onSort={setSort}
              query={query}
              onQuery={setQuery}
            />
            <div className="content">
              <div className="section-head">
                <h3>{categoryLabel}</h3>
                <span className="hint">
                  {total.toLocaleString()} themes · loaded {themes.length}
                </span>
              </div>
              <ThemeGrid
                themes={themes}
                installed={installed}
                favorites={favorites}
                onOpen={openDetail}
                onApply={openApply}
                onToggleFavorite={toggleFavorite}
              />
              {loading && themes.length === 0 && (
                <div className="empty">Loading…</div>
              )}
              {!loading && themes.length === 0 && (
                <div className="empty">No themes found.</div>
              )}
              <div ref={sentinelRef} className="load-more">
                {loading && themes.length > 0 && (
                  <span className="hint">Loading more…</span>
                )}
              </div>
            </div>
          </>
        )}

        {view === "installed" && (
          <div className="content">
            <div className="section-head">
              <h3>Installed</h3>
              <span className="hint">{installed.size} themes</span>
            </div>
            {installed.size === 0 ? (
              <div className="empty">Nothing installed yet. Apply a theme to see it here.</div>
            ) : (
              <ThemeGrid
                themes={[...installed.values()]}
                installed={installed}
                favorites={favorites}
                onOpen={openDetail}
                onApply={openApply}
                onToggleFavorite={toggleFavorite}
              />
            )}
          </div>
        )}

        {view === "favorites" && (
          <div className="content">
            <div className="section-head">
              <h3>Favorites</h3>
              <span className="hint">{favorites.size} themes</span>
            </div>
            {favorites.size === 0 ? (
              <div className="empty">No favorites yet. Tap the ♡ on any theme.</div>
            ) : (
              <ThemeGrid
                themes={[...favorites.values()]}
                installed={installed}
                favorites={favorites}
                onOpen={openDetail}
                onApply={openApply}
                onToggleFavorite={toggleFavorite}
              />
            )}
          </div>
        )}

        {view === "detail" && selected && (
          <ThemeDetail
            theme={selected}
            installed={installed.has(selected.id)}
            favorite={favorites.has(selected.id)}
            onBack={() => setView(prevView)}
            onApply={openApply}
            onToggleFavorite={toggleFavorite}
          />
        )}
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
