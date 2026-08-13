import type { Theme } from "../types/theme";
import { wallpaperBackground, fmtDownloads } from "../lib/format";
import { Stars } from "./Stars";
import { DownloadIcon, CheckIcon } from "./Icon";

interface Props {
  theme: Theme;
  installed: boolean;
  onApply: (t: Theme) => void;
}

export function ThemeCard({ theme, installed, onApply }: Props) {
  return (
    <article className="card">
      <div
        className="card-thumb"
        style={{ background: wallpaperBackground(theme) }}
        onClick={() => onApply(theme)}
      >
        {theme.preview && (
          <img className="thumb-img" src={theme.preview} alt={theme.name} loading="lazy" />
        )}
        {installed && <span className="badge active">Active</span>}
      </div>
      <div className="card-body">
        <div className="card-title-row">
          <h4>{theme.name}</h4>
          {theme.palette && (
            <div className="swatches">
              {theme.palette.slice(0, 5).map((c) => (
                <span key={c} className="swatch" style={{ background: c }} />
              ))}
            </div>
          )}
        </div>
        <div className="card-desc">{theme.description}</div>
        <div className="card-meta">
          by {theme.author} · {theme.category}
        </div>
        <div className="card-footer">
          <div className="card-stats">
            <Stars rating={theme.rating} />
            <span className="downloads">↓ {fmtDownloads(theme.downloads)}</span>
          </div>
          <button
            className={`btn btn-ghost ${installed ? "installed" : ""}`}
            onClick={() => onApply(theme)}
          >
            {installed ? <CheckIcon /> : <DownloadIcon />}
            {installed ? "Applied" : "Apply"}
          </button>
        </div>
      </div>
    </article>
  );
}
