import type { Theme } from "../types/theme";
import { ThemeCard } from "./ThemeCard";

interface Props {
  themes: Theme[];
  installed: Set<string>;
  onApply: (t: Theme) => void;
}

export function ThemeGrid({ themes, installed, onApply }: Props) {
  return (
    <div className="grid">
      {themes.map((t) => (
        <ThemeCard
          key={t.id}
          theme={t}
          installed={installed.has(t.id)}
          onApply={onApply}
        />
      ))}
    </div>
  );
}
