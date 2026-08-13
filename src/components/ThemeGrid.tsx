import type { Theme } from "../types/theme";
import { ThemeCard } from "./ThemeCard";

interface Props {
  themes: Theme[];
  installed: Map<string, Theme>;
  favorites: Map<string, Theme>;
  onOpen: (t: Theme) => void;
  onApply: (t: Theme) => void;
  onToggleFavorite: (t: Theme) => void;
}

export function ThemeGrid({
  themes,
  installed,
  favorites,
  onOpen,
  onApply,
  onToggleFavorite,
}: Props) {
  return (
    <div className="grid">
      {themes.map((t) => (
        <ThemeCard
          key={t.id}
          theme={t}
          installed={installed.has(t.id)}
          favorite={favorites.has(t.id)}
          onOpen={onOpen}
          onApply={onApply}
          onToggleFavorite={onToggleFavorite}
        />
      ))}
    </div>
  );
}
