import { useEffect, useMemo, useState } from "react";
import type { InstalledTheme } from "../types/theme";
import { applyComponent } from "../lib/installed";
import { fetchCurrentTheme } from "../lib/studio";
import { applyAccent } from "../lib/accent";

interface Props {
  installed: InstalledTheme[];
}

const ACCENTS = [
  { hex: "#3daee9", name: "Breeze Blue" },
  { hex: "#3ddc97", name: "Green" },
  { hex: "#f0a13d", name: "Amber" },
  { hex: "#e93d3d", name: "Red" },
  { hex: "#b26bf0", name: "Purple" },
  { hex: "#3dd3d3", name: "Cyan" },
  { hex: "#f05a9c", name: "Pink" },
  { hex: "#8a8a8a", name: "Gray" },
];

const SECTIONS = [
  { kind: "gtk", match: "gtk", label: "GTK Theme" },
  { kind: "icons", match: "icons", label: "Icon Theme" },
  { kind: "cursors", match: "cursors", label: "Cursor Theme" },
  { kind: "colors", match: "colors", label: "Color Scheme" },
  { kind: "plasma", match: "plasma", label: "Plasma Theme" },
];

export function Settings({ installed }: Props) {
  const [current, setCurrent] = useState<Record<string, string>>({});
  const [draft, setDraft] = useState<Record<string, string>>({});
  const [accent, setAccent] = useState("#3daee9");
  const [appliedAccent, setAppliedAccent] = useState("#3daee9");
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    fetchCurrentTheme()
      .then((c) => {
        const cur = {
          gtk: c.gtkTheme,
          icons: c.iconTheme,
          cursors: c.cursorTheme,
          colors: c.colorScheme,
          plasma: c.plasmaTheme,
        };
        setCurrent(cur);
        setDraft(cur);
        const a = c.accentColor || "#3daee9";
        setAccent(a);
        setAppliedAccent(a);
        applyAccent(a);
      })
      .catch(() => {});
  }, []);

  const dirty = useMemo(() => {
    for (const s of SECTIONS) {
      if ((draft[s.kind] ?? "") !== (current[s.kind] ?? "")) return true;
    }
    return accent !== appliedAccent;
  }, [draft, current, accent, appliedAccent]);

  const options = (match: string, cur: string) => {
    const set = new Set(installed.filter((t) => t.kind === match).map((t) => t.name));
    if (cur) set.add(cur); // always show the applied value, even if not installed
    return [...set].sort((a, b) => a.localeCompare(b));
  };

  const apply = async () => {
    setBusy(true);
    try {
      for (const s of SECTIONS) {
        const v = draft[s.kind] ?? "";
        if (v && v !== (current[s.kind] ?? "")) {
          await applyComponent(s.kind, v);
        }
      }
      if (accent !== appliedAccent) {
        await applyComponent("accent", accent);
        applyAccent(accent);
        setAppliedAccent(accent);
      }
      setCurrent(draft);
      setNotice("Applied ✓");
    } catch (e) {
      setNotice(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="content">
      <div className="section-head">
        <h3>Settings</h3>
        <span className="hint">Pick components, then Apply</span>
      </div>

      {notice && <div className="notice">{notice}</div>}

      <div className="studio-label">Accent color</div>
      <div className="accent-row">
        {ACCENTS.map((a) => (
          <button
            key={a.hex}
            className={`accent-swatch ${accent.toLowerCase() === a.hex ? "on" : ""}`}
            style={{ background: a.hex }}
            title={a.name}
            onClick={() => setAccent(a.hex)}
          />
        ))}
      </div>

      {SECTIONS.map((s) => (
        <div key={s.kind} className="settings-field">
          <label className="settings-label">{s.label}</label>
          <select
            className="settings-select"
            value={draft[s.kind] ?? ""}
            onChange={(e) => setDraft((d) => ({ ...d, [s.kind]: e.target.value }))}
          >
            <option value="">— none —</option>
            {options(s.match, current[s.kind]).map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
        </div>
      ))}

      {dirty && (
        <div className="settings-apply">
          <button className="btn btn-primary" onClick={apply} disabled={busy}>
            {busy ? "Applying…" : "Apply changes"}
          </button>
        </div>
      )}
    </div>
  );
}
