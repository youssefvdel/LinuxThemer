import type { InstalledTheme } from "../types/theme";

export function InstalledCard({ item }: { item: InstalledTheme }) {
  return (
    <div className="installed-card">
      <div className="installed-name">{item.name}</div>
      <div className="installed-path">{item.path}</div>
    </div>
  );
}
