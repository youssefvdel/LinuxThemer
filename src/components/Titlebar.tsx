import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Square, X } from "lucide-react";
import logo from "../assets/logo.png";

const hasTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export function Titlebar() {
  const win = hasTauri ? getCurrentWindow() : null;
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    if (!win) return;
    let disposed = false;
    win.isMaximized().then((m) => {
      if (!disposed) setMaximized(m);
    });
    const unlisten = win.onResized(() => {
      win.isMaximized().then((m) => {
        if (!disposed) setMaximized(m);
      });
    });
    return () => {
      disposed = true;
      unlisten.then((f) => f());
    };
  }, [win]);

  return (
    <div className="titlebar" data-tauri-drag-region>
      <div className="titlebar-brand">
        <img src={logo} alt="" />
        <span>LinuxThemer</span>
      </div>
      <div className="titlebar-controls">
        <button aria-label="Minimize" onClick={() => win?.minimize()}>
          <Minus size={14} />
        </button>
        <button aria-label="Maximize" onClick={() => win?.toggleMaximize()}>
          {maximized ? <Copy size={12} /> : <Square size={12} />}
        </button>
        <button className="close" aria-label="Close" onClick={() => win?.close()}>
          <X size={14} />
        </button>
      </div>
    </div>
  );
}
