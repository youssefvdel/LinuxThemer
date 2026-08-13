import { useEffect, useMemo, useState } from "react";
import type { GlobalThemeSpec, InstalledTheme } from "../types/theme";
import { fetchCurrentTheme, saveGlobalTheme } from "../lib/studio";

interface Props {
  installed: InstalledTheme[];
  onSaved: () => void;
}

type TabId = "gtk" | "qt" | "icons" | "cursors" | "colors" | "plasma";

const TABS: { id: TabId; label: string }[] = [
  { id: "gtk", label: "GTK" },
  { id: "qt", label: "Qt / Kvantum" },
  { id: "icons", label: "Icons" },
  { id: "cursors", label: "Cursors" },
  { id: "colors", label: "Colors" },
  { id: "plasma", label: "Plasma" },
];

function emptySpec(): GlobalThemeSpec {
  return {
    gtk: "",
    widgetStyle: "Breeze",
    kvantum: "",
    icons: "",
    cursors: "",
    colors: "",
    plasma: "",
  };
}

export function Studio({ installed, onSaved }: Props) {
  const [spec, setSpec] = useState<GlobalThemeSpec | null>(null);
  const [tab, setTab] = useState<TabId>("gtk");
  const [name, setName] = useState("");
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState("");

  useEffect(() => {
    fetchCurrentTheme()
      .then((c) =>
        setSpec({
          gtk: c.gtkTheme,
          widgetStyle: c.widgetStyle || "Breeze",
          kvantum: c.kvantum,
          icons: c.iconTheme,
          cursors: c.cursorTheme,
          colors: c.colorScheme,
          plasma: c.plasmaTheme,
        })
      )
      .catch(() => setSpec(emptySpec()));
  }, []);

  const opt = useMemo(() => {
    const by = (k: string) => installed.filter((t) => t.kind === k);
    return {
      gtk: by("gtk"),
      icons: by("icons"),
      cursors: by("cursors"),
      colors: by("colors"),
      plasma: by("plasma"),
      kvantum: by("kvantum"),
    };
  }, [installed]);

  const set = (patch: Partial<GlobalThemeSpec>) =>
    setSpec((s) => (s ? { ...s, ...patch } : s));

  const save = async () => {
    const n = name.trim();
    if (!n || !spec || saving) return;
    setSaving(true);
    setNotice("");
    try {
      const p = await saveGlobalTheme(n, spec);
      setNotice(`Saved global theme “${n}” — ${p}`);
      setName("");
      onSaved();
    } catch (e) {
      setNotice(String(e));
    } finally {
      setSaving(false);
    }
  };

  if (!spec) {
    return (
      <div className="content">
        <div className="empty">Reading current theme…</div>
      </div>
    );
  }

  const current = (id: TabId): string => {
    switch (id) {
      case "gtk":
        return spec.gtk;
      case "qt":
        return spec.widgetStyle === "kvantum"
          ? `Kvantum — ${spec.kvantum || "default"}`
          : spec.widgetStyle;
      case "icons":
        return spec.icons;
      case "cursors":
        return spec.cursors;
      case "colors":
        return spec.colors;
      case "plasma":
        return spec.plasma;
    }
  };

  return (
    <div className="content">
      <div className="section-head">
        <h3>Theme Studio</h3>
        <span className="hint">Build a global theme from your current one</span>
      </div>

      <div className="save-bar studio-save">
        <input
          placeholder="New theme name…"
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && save()}
        />
        <button
          className="btn btn-primary"
          onClick={save}
          disabled={!name.trim() || saving}
        >
          {saving ? "Saving…" : "Save global theme"}
        </button>
      </div>

      {notice && <div className="notice">{notice}</div>}

      <div className="tabs">
        {TABS.map((t) => (
          <button
            key={t.id}
            className={`tab ${tab === t.id ? "active" : ""}`}
            onClick={() => setTab(t.id)}
          >
            {t.label}
          </button>
        ))}
      </div>

      <div className="studio-current">
        Current: <strong>{current(tab) || "—"}</strong>
      </div>

      {tab === "gtk" && (
        <PickList
          label="GTK themes"
          options={opt.gtk.map((t) => t.name)}
          current={spec.gtk}
          onPick={(v) => set({ gtk: v })}
        />
      )}

      {tab === "qt" && (
        <>
          <div className="studio-label">Widget style</div>
          <div className="studio-pills">
            {["Breeze", "Fusion", "kvantum"].map((s) => (
              <button
                key={s}
                className={`studio-pill ${spec.widgetStyle === s ? "on" : ""}`}
                onClick={() => set({ widgetStyle: s })}
              >
                {s}
              </button>
            ))}
          </div>
          {spec.widgetStyle === "kvantum" && (
            <PickList
              label="Kvantum themes"
              options={opt.kvantum.map((t) => t.name)}
              current={spec.kvantum}
              onPick={(v) => set({ kvantum: v })}
            />
          )}
        </>
      )}

      {tab === "icons" && (
        <PickList
          label="Icon themes"
          options={opt.icons.map((t) => t.name)}
          current={spec.icons}
          onPick={(v) => set({ icons: v })}
          allowNone
        />
      )}

      {tab === "cursors" && (
        <PickList
          label="Cursor themes"
          options={opt.cursors.map((t) => t.name)}
          current={spec.cursors}
          onPick={(v) => set({ cursors: v })}
        />
      )}

      {tab === "colors" && (
        <PickList
          label="Color schemes"
          options={opt.colors.map((t) => t.name)}
          current={spec.colors}
          onPick={(v) => set({ colors: v })}
        />
      )}

      {tab === "plasma" && (
        <PickList
          label="Plasma desktop themes"
          options={opt.plasma.map((t) => t.name)}
          current={spec.plasma}
          onPick={(v) => set({ plasma: v })}
        />
      )}
    </div>
  );
}

function PickList({
  label,
  options,
  current,
  onPick,
  allowNone,
}: {
  label: string;
  options: string[];
  current: string;
  onPick: (v: string) => void;
  allowNone?: boolean;
}) {
  if (options.length === 0) {
    return (
      <div className="notice">No installed {label.toLowerCase()} found on this device.</div>
    );
  }
  return (
    <div className="studio-block">
      <div className="studio-label">{label}</div>
      <div className="studio-grid">
        {allowNone && (
          <button
            className={`studio-option ${current === "" ? "on" : ""}`}
            onClick={() => onPick("")}
          >
            <span className="studio-option-name">System default</span>
          </button>
        )}
        {options.map((o) => (
          <button
            key={o}
            className={`studio-option ${current === o ? "on" : ""}`}
            onClick={() => onPick(o)}
          >
            <span className="studio-option-name">{o}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
