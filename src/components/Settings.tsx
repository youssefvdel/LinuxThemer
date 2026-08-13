import { useEffect, useState } from "react";
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
  const [accent, setAccent] = useState("#3daee9");
  const [current, setCurrent] = useState<Record<string, string>>({});
  const [notice, setNotice] = useState("");

  useEffect(() => {
    fetchCurrentTheme()
      .then((c) => {
        setAccent(c.accentColor || "#3daee9");
        applyAccent(c.accentColor || "#3daee9");
        setCurrent({
          gtk: c.gtkTheme,
          icons: c.iconTheme,
          cursors: c.cursorTheme,
          colors: c.colorScheme,
          plasma: c.plasmaTheme,
        });
      })
      .catch(() => {});
  }, []);

  const pickAccent = async (hex: string) => {
    setAccent(hex);
    applyAccent(hex);
    try {
      await applyComponent("accent", hex);
      setNotice(`Accent → ${hex}`);
    } catch (e) {
      setNotice(String(e));
    }
  };

  const pick = async (kind: string, value: string) => {
    setCurrent((c) => ({ ...c, [kind]: value }));
    try {
      await applyComponent(kind, value);
      setNotice(`Applied ${kind} → ${value}`);
    } catch (e) {
      setNotice(String(e));
    }
  };

  const byKind = (k: string) => installed.filter((t) => t.kind === k);

  return (
    <div className="content">
      <div className="section-head">
        <h3>Settings</h3>
        <span className="hint">Accent color + live system themes</span>
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
            onClick={() => pickAccent(a.hex)}
          />
        ))}
      </div>

      {SECTIONS.map((s) => (
        <div key={s.kind} className="studio-block">
          <div className="studio-label">
            {s.label} <span className="settings-val">— {current[s.kind] || "none"}</span>
          </div>
          <div className="studio-grid">
            {byKind(s.match).map((t) => (
              <button
                key={t.id}
                className={`studio-option ${current[s.kind] === t.name ? "on" : ""}`}
                onClick={() => pick(s.kind, t.name)}
              >
                <span className="studio-option-name">{t.name}</span>
              </button>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
