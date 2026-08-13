import type { Theme } from "../types/theme";
import { APPLY_COMPONENTS } from "../types/theme";
import { CheckIcon } from "./Icon";

interface Props {
  theme: Theme;
  selected: Set<string>;
  onToggle: (id: string) => void;
  onCancel: () => void;
  onConfirm: () => void;
}

export function ApplyModal({ theme, selected, onToggle, onCancel, onConfirm }: Props) {
  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-head">
          <h3>Apply {theme.name}</h3>
          <button className="modal-close" onClick={onCancel}>
            ×
          </button>
        </div>
        <p className="modal-sub">Choose what to apply. Unchecked parts stay untouched.</p>
        <div className="apply-list">
          {APPLY_COMPONENTS.map((c) => (
            <label key={c.id} className="apply-row">
              <span>{c.label}</span>
              <input
                type="checkbox"
                checked={selected.has(c.id)}
                onChange={() => onToggle(c.id)}
              />
            </label>
          ))}
        </div>
        <div className="modal-actions">
          <button className="btn btn-ghost" onClick={onCancel}>
            Cancel
          </button>
          <button className="btn btn-primary" onClick={onConfirm}>
            <CheckIcon />
            Apply
          </button>
        </div>
      </div>
    </div>
  );
}
