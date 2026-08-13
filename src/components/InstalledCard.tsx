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

export function InstalledCard({ item, active, onApply, onRemove }: Props) {
  const [confirming, setConfirming] = useState(false);
  const kindLabel = INSTALLED_KIND_LABELS[item.kind] ?? item.kind;
  const src = item.preview ? convertFileSrc(item.preview) : null;

  return (
    <div className="installed-card">
      <div className="installed-thumb">
        {src ? (
          <img src={src} alt={item.name} loading="lazy" />
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
