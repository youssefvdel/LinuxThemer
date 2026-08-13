import { useEffect, useState } from "react";
import type { Theme } from "../types/theme";
import { wallpaperBackground, fmtDownloads } from "../lib/format";
import { Stars } from "./Stars";
import { DownloadIcon, CheckIcon, HeartIcon } from "./Icon";

interface Props {
  theme: Theme;
  installed: boolean;
  favorite: boolean;
  onOpen: (t: Theme) => void;
  onApply: (t: Theme) => void;
  onToggleFavorite: (t: Theme) => void;
}

export function ThemeCard({
  theme,
  installed,
  favorite,
  onOpen,
  onApply,
  onToggleFavorite,
}: Props) {
  const images = theme.images?.length
    ? theme.images
    : theme.preview
      ? [theme.preview]
      : [];
  const [hovering, setHovering] = useState(false);
  const [idx, setIdx] = useState(0);

  // Cycle through preview images while hovered.
  useEffect(() => {
    if (!hovering || images.length < 2) return;
    const t = setInterval(() => setIdx((i) => (i + 1) % images.length), 1600);
    return () => clearInterval(t);
  }, [hovering, images.length]);

  const current = images[idx];

  return (
    <article
      className="card"
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => {
        setHovering(false);
        setIdx(0);
      }}
      onClick={() => onOpen(theme)}
    >
      <div className="card-thumb" style={{ background: wallpaperBackground(theme) }}>
        {current && <img className="thumb-img" src={current} alt={theme.name} loading="lazy" />}
        {images.length > 1 && (
          <div className="img-dots">
            {images.map((_, i) => (
              <span key={i} className={`dot ${i === idx ? "on" : ""}`} />
            ))}
          </div>
        )}
        <button
          className={`fav-btn ${favorite ? "on" : ""}`}
          onClick={(e) => {
            e.stopPropagation();
            onToggleFavorite(theme);
          }}
          aria-label="Favorite"
        >
          <HeartIcon filled={favorite} />
        </button>
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
            onClick={(e) => {
              e.stopPropagation();
              onApply(theme);
            }}
          >
            {installed ? <CheckIcon /> : <DownloadIcon />}
            {installed ? "Applied" : "Apply"}
          </button>
        </div>
      </div>
    </article>
  );
}
