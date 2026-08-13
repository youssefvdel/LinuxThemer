import { useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { InstalledTheme } from "../types/theme";
import { INSTALLED_KIND_LABELS } from "../types/theme";
import { CheckIcon, TrashIcon } from "./Icon";

interface Props {
  item: InstalledTheme;
  active: boolean;
  onApply: (item: InstalledTheme) => void;
  onRemove: (item: InstalledTheme) => void;
}

/** Pick black/white text color that reads on a given hex background. */
function contrast(hex: string): string {
  const h = hex.replace("#", "");
  if (h.length !== 6) return "#fff";
  const r = parseInt(h.slice(0, 2), 16);
  const g = parseInt(h.slice(2, 4), 16);
  const b = parseInt(h.slice(4, 6), 16);
  return 0.299 * r + 0.587 * g + 0.114 * b > 150 ? "#1c1c1c" : "#f2f2f2";
}

/** KDE-System-Settings-style mock window, colored from the theme palette. */
function ColorMock({ palette }: { palette: string[] }) {
  const win = palette[0] ?? "#3a3a3a";
  const view = palette[1] ?? win;
  const btn = palette[2] ?? win;
  const sel = palette[3] ?? "#3daee9";
  const fg = palette[4] ?? contrast(win);
  const vfg = palette[5] ?? fg;
  const link = palette[2] ?? "#3daee9";
  return (
    <div className="mock-win" style={{ background: win, color: fg }}>
      <div className="mock-titlebar">
        <span className="mock-dot" style={{ background: sel }} />
        Window title
      </div>
      <div className="mock-body" style={{ background: view, color: vfg }}>
        <span className="mock-line">Normal text</span>
        <span className="mock-link" style={{ color: link }}>
          link
        </span>
        <span className="mock-button" style={{ background: btn, color: contrast(btn) }}>
          Button
        </span>
        <span className="mock-highlight" style={{ background: sel, color: contrast(sel) }}>
          Highlighted text
        </span>
        <span className="mock-line mock-disabled">Disabled text</span>
      </div>
    </div>
  );
}

/** KDE-System-Settings-style window mockup for window-decoration themes. */
function DecorationMock({ buttons, titlebar }: { buttons: string[]; titlebar?: string }) {
  const tb = titlebar ?? "#2a2d34";
  return (
    <div className="deco-win">
      <div className="deco-back" style={{ background: tb }} />
      <div className="deco-front">
        <div className="deco-titlebar" style={{ background: tb, color: contrast(tb) }}>
          <div className="deco-buttons">
            {buttons.map((b) => (
              <img key={b} src={convertFileSrc(b)} alt="" />
            ))}
          </div>
          <span className="deco-title">Window</span>
        </div>
        <div className="deco-body" />
      </div>
    </div>
  );
}

export function InstalledCard({ item, active, onApply, onRemove }: Props) {
  const [confirming, setConfirming] = useState(false);
  const kindLabel = INSTALLED_KIND_LABELS[item.kind] ?? item.kind;
  const src = item.preview ? convertFileSrc(item.preview) : null;
  const isVideo = !!item.preview && /\.(mp4|webm)$/i.test(item.preview);
  const samples = item.samples ?? [];
  const palette = item.palette ?? [];
  const mockWindow = palette.length > 0 && (item.kind === "colors" || item.kind === "gtk");

  return (
    <div className="installed-card">
      <div className="installed-thumb">
        {src ? (
          isVideo ? (
            <video src={src} muted loop playsInline preload="metadata" />
          ) : (
            <img src={src} alt={item.name} decoding="async" />
          )
        ) : item.kind === "cursors" && samples.length ? (
          <div className="cursor-box">
            {samples.map((s) => (
              <img key={s} src={convertFileSrc(s)} alt="" decoding="async" />
            ))}
          </div>
        ) : item.kind === "icons" && samples.length ? (
          <div className="sample-row">
            {samples.map((s) => (
              <img key={s} src={convertFileSrc(s)} alt="" decoding="async" />
            ))}
          </div>
        ) : item.kind === "decorations" && samples.length ? (
          <DecorationMock buttons={samples} titlebar={palette[0]} />
        ) : mockWindow ? (
          <ColorMock palette={palette} />
        ) : palette.length ? (
          <div className="installed-palette">
            {palette.map((c) => (
              <span key={c} className="palette-chip" style={{ background: c }} />
            ))}
          </div>
        ) : (
          <span className="installed-thumb-fallback">{kindLabel}</span>
        )}
        {active && <span className="badge active">Active</span>}
      </div>
      <div className="installed-body">
        <div className="installed-name">{item.name}</div>
        <div className="installed-meta">{kindLabel}</div>
        <div className="installed-path">{item.path}</div>
        <div className="installed-actions">
          <button className="btn btn-primary" onClick={() => onApply(item)}>
            <CheckIcon /> Apply
          </button>
          <button
            className={`btn btn-ghost ${confirming ? "danger" : ""}`}
            onClick={() => {
              if (confirming) {
                setConfirming(false);
                onRemove(item);
              } else {
                setConfirming(true);
              }
            }}
          >
            <TrashIcon /> {confirming ? "Confirm" : "Remove"}
          </button>
        </div>
      </div>
    </div>
  );
}
