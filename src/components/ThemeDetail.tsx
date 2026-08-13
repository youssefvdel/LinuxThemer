import { useState } from "react";
import type { Theme } from "../types/theme";
import { wallpaperBackground, fmtDownloads } from "../lib/format";
import { Stars } from "./Stars";
import { DownloadIcon, CheckIcon, HeartIcon, BackIcon } from "./Icon";

interface Props {
  theme: Theme;
  installed: boolean;
  favorite: boolean;
  onBack: () => void;
  onApply: (t: Theme) => void;
  onToggleFavorite: (t: Theme) => void;
}

export function ThemeDetail({
  theme,
  installed,
  favorite,
  onBack,
  onApply,
  onToggleFavorite,
}: Props) {
  const images = theme.images?.length
    ? theme.images
    : theme.preview
      ? [theme.preview]
      : [];
  const [idx, setIdx] = useState(0);

  return (
    <div className="content detail">
      <button className="back-btn" onClick={onBack}>
        <BackIcon /> Back
      </button>

      <div className="detail-hero">
        <div className="detail-gallery">
          <div className="detail-img" style={{ background: wallpaperBackground(theme) }}>
            {images[idx] && <img src={images[idx]} alt={theme.name} />}
          </div>
          {images.length > 1 && (
            <div className="detail-thumbs">
              {images.map((src, i) => (
                <button
                  key={i}
                  className={`dthumb ${i === idx ? "on" : ""}`}
                  onClick={() => setIdx(i)}
                >
                  <img src={src} alt="" loading="lazy" />
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="detail-info">
          <div className="detail-tags">
            {theme.tags.map((t) => (
              <span key={t} className="tag">
                {t}
              </span>
            ))}
          </div>
          <h1>{theme.name}</h1>
          <div className="detail-by">
            by <strong>{theme.author}</strong> · {theme.category}
          </div>
          <div className="detail-meta">
            <Stars rating={theme.rating} />
            <span className="downloads">
              <DownloadIcon size={13} /> {fmtDownloads(theme.downloads)}
            </span>
          </div>
          <p className="detail-desc">{theme.description}</p>
          <div className="detail-actions">
            <button
              className={`btn ${installed ? "btn-ghost installed" : "btn-primary"}`}
              onClick={() => onApply(theme)}
            >
              {installed ? <CheckIcon /> : <DownloadIcon />}
              {installed ? "Applied" : "Apply theme"}
            </button>
            <button
              className={`btn btn-ghost ${favorite ? "faved" : ""}`}
              onClick={() => onToggleFavorite(theme)}
            >
              <HeartIcon filled={favorite} />
              {favorite ? "Favorited" : "Favorite"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
