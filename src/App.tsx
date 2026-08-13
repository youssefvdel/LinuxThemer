import { useEffect, useMemo, useRef, useState } from "react";
import { fetchStore, STORE_CATEGORIES } from "./lib/store";
import { fetchInstalled, saveCurrentTheme, launchStudio } from "./lib/installed";
import {
  APPLY_COMPONENTS,
  INSTALLED_KIND_LABELS,
  type InstalledTheme,
  type SortId,
  type Theme,
  type View,
} from "./types/theme";
import { Sidebar } from "./components/Sidebar";
import { Topbar } from "./components/Topbar";
import { ThemeGrid } from "./components/ThemeGrid";
import { ApplyModal } from "./components/ApplyModal";
import { ThemeDetail } from "./components/ThemeDetail";
import { InstalledCard } from "./components/InstalledCard";

function loadThemes(key: string): Map<string, Theme> {
  try {
    const arr = JSON.parse(localStorage.getItem(key) ?? "[]") as Theme[];
    return new Map(arr.map((t) => [t.id, t]));
  } catch {
    return new Map();
  }
}

const INSTALLED_ORDER = [
  "global",
  "gtk",
  "plasma",
  "icons",
  "cursors",
  "decorations",
  "colors",
  "sddm",
  "wallpapers",
  "kvantum",
  "custom",
];

const STUDIO_TOOLS = [
  { kind: "gtk", label: "GTK Theme Creator", desc: "Oomox — design & recolor GTK3/GTK4 themes" },
  { kind: "plasma", label: "Plasma Look & Feel", desc: "KDE settings — global theme, splash, lockscreen" },
  { kind: "colors", label: "Plasma Color Schemes", desc: "KDE settings — color scheme editor" },
  { kind: "cursors", label: "Cursor Themes", desc: "KDE settings — cursor theme picker" },
  { kind: "icons", label: "Icon Themes", desc: "KDE settings — icon theme picker" },
];

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
  const [installedList, setInstalledList] = useState<InstalledTheme[]>([]);
  const [installedTab, setInstalledTab] = useState("global");
  const [showSave, setShowSave] = useState(false);
  const [saveName, setSaveName] = useState("");
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState("");

  const sentinelRef = useRef<HTMLDivElement | null>(null);
  const mainRef = useRef<HTMLDivElement | null>(null);

  // Persist installed + favorites (full theme objects, not just ids).
  useEffect(() => {
    localStorage.setItem("lt.installed", JSON.stringify([...installed.values()]));
  }, [installed]);
  useEffect(() => {
    localStorage.setItem("lt.favorites", JSON.stringify([...favorites.values()]));
  }, [favorites]);

  // Device scan: once on mount, then refresh whenever entering Installed view.
  useEffect(() => {
    fetchInstalled().then(setInstalledList).catch(() => setInstalledList([]));
  }, []);
  useEffect(() => {
    if (view !== "installed") return;
    fetchInstalled().then(setInstalledList).catch(() => setInstalledList([]));
  }, [view]);

  // Scroll to top whenever the view / category / installed tab changes.
  useEffect(() => {
    mainRef.current?.querySelector(".content")?.scrollTo({ top: 0 });
  }, [view, category, installedTab]);

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

  const grouped = useMemo(() => {
    const m = new Map<string, InstalledTheme[]>();
    for (const t of installedList) {
      const arr = m.get(t.kind) ?? [];
      arr.push(t);
      m.set(t.kind, arr);
    }
    return m;
  }, [installedList]);

  const installedKinds = useMemo(() => {
    const present = new Set(grouped.keys());
    const ordered = INSTALLED_ORDER.filter((k) => present.has(k));
    const extra = [...present].filter((k) => !INSTALLED_ORDER.includes(k));
    return [...ordered, ...extra];
  }, [grouped]);

  const doSave = async () => {
    const name = saveName.trim();
    if (!name || saving) return;
    setSaving(true);
    setNotice("");
    try {
      await saveCurrentTheme(name);
      setSaveName("");
      setShowSave(false);
      const list = await fetchInstalled();
      setInstalledList(list);
      setInstalledTab("custom");
    } catch (e) {
      setNotice(String(e));
    } finally {
      setSaving(false);
    }
  };

  const launch = async (kind: string) => {
    setNotice("");
    try {
      await launchStudio(kind);
      setNotice("Launched.");
    } catch (e) {
      setNotice(String(e));
    }
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
        installedCount={installedList.length}
        favoritesCount={favorites.size}
      />
      <div className="main" ref={mainRef}>
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
              <span className="hint">{installedList.length} on this device</span>
              <div className="head-actions">
                {showSave ? (
                  <div className="save-bar">
                    <input
                      autoFocus
                      placeholder="Theme name…"
                      value={saveName}
                      onChange={(e) => setSaveName(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") doSave();
                        if (e.key === "Escape") setShowSave(false);
                      }}
                    />
                    <button
                      className="btn btn-primary"
                      onClick={doSave}
                      disabled={!saveName.trim() || saving}
                    >
                      {saving ? "Saving…" : "Save"}
                    </button>
                    <button className="btn btn-ghost" onClick={() => setShowSave(false)}>
                      Cancel
                    </button>
                  </div>
                ) : (
                  <button className="btn btn-ghost" onClick={() => setShowSave(true)}>
                    Save current theme
                  </button>
                )}
              </div>
            </div>
            {notice && <div className="notice">{notice}</div>}
            {installedKinds.length === 0 ? (
              <div className="empty">No themes found on this device.</div>
            ) : (
              <>
                <div className="tabs">
                  {installedKinds.map((k) => (
                    <button
                      key={k}
                      className={`tab ${installedTab === k ? "active" : ""}`}
                      onClick={() => setInstalledTab(k)}
                    >
                      {INSTALLED_KIND_LABELS[k] ?? k}
                      <span className="tab-count">{grouped.get(k)?.length ?? 0}</span>
                    </button>
                  ))}
                </div>
                <div className="installed-grid">
                  {(grouped.get(installedTab) ?? []).map((it) => (
                    <InstalledCard key={it.id} item={it} />
                  ))}
                </div>
              </>
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
              <div className="empty">No favorites yet. Tap the heart on any theme.</div>
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

        {view === "studio" && (
          <div className="content">
            <div className="section-head">
              <h3>Theme Studio</h3>
              <span className="hint">Open the desktop's own theme creators</span>
            </div>
            {notice && <div className="notice">{notice}</div>}
            <div className="studio-grid">
              {STUDIO_TOOLS.map((t) => (
                <button key={t.kind} className="studio-card" onClick={() => launch(t.kind)}>
                  <div className="studio-name">{t.label}</div>
                  <div className="studio-desc">{t.desc}</div>
                </button>
              ))}
            </div>
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
