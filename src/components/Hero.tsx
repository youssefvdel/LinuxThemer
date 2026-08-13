import type { Theme } from "../types/theme";
import { wallpaperBackground, fmtDownloads } from "../lib/format";
import { Stars } from "./Stars";
import { DownloadIcon, CheckIcon } from "./Icon";

interface Props {
  theme: Theme;
  installed: boolean;
  onApply: (t: Theme) => void;
}

export function Hero({ theme, installed, onApply }: Props) {
  return (
    <div className="hero">
      <div className="hero-art" style={{ background: wallpaperBackground(theme) }} />
      <div className="hero-body">
        <span className="hero-tag">Featured</span>
        <h2>{theme.name}</h2>
        <p>{theme.description}</p>
        <div className="hero-meta">
          <Stars rating={theme.rating} />
          <span className="downloads">↓ {fmtDownloads(theme.downloads)}</span>
        </div>
        <div className="hero-actions">
          <button
            className={`btn ${installed ? "btn-ghost installed" : "btn-primary"}`}
            onClick={() => onApply(theme)}
          >
            {installed ? <CheckIcon /> : <DownloadIcon />}
            {installed ? "Applied" : "Apply theme"}
          </button>
          <button className="btn btn-ghost">Preview</button>
        </div>
      </div>
    </div>
  );
}
