import { useState } from "react";
import { SearchIcon, ChevronIcon } from "./Icon";
import type { SortId } from "../types/theme";

const SORTS: { id: SortId; label: string }[] = [
  { id: "popular", label: "Most downloaded" },
  { id: "rating", label: "Top rated" },
  { id: "name", label: "Name A–Z" },
];

interface Props {
  count: number;
  sort: SortId;
  onSort: (s: SortId) => void;
  query: string;
  onQuery: (q: string) => void;
}

export function Topbar({ count, sort, onSort, query, onQuery }: Props) {
  const [open, setOpen] = useState(false);
  return (
    <header className="topbar">
      <h1>
        Browse themes <span className="sub">· {count} shown</span>
      </h1>
      <div className="sort">
        <button className="sort-btn" onClick={() => setOpen((o) => !o)}>
          {SORTS.find((s) => s.id === sort)?.label}
          <ChevronIcon />
        </button>
        {open && (
          <div className="sort-menu">
            {SORTS.map((s) => (
              <button
                key={s.id}
                className={sort === s.id ? "active" : ""}
                onClick={() => {
                  onSort(s.id);
                  setOpen(false);
                }}
              >
                {s.label}
              </button>
            ))}
          </div>
        )}
      </div>
      <div className="search">
        <SearchIcon />
        <input
          placeholder="Search themes, authors, tags…"
          value={query}
          onChange={(e) => onQuery(e.target.value)}
        />
      </div>
    </header>
  );
}
