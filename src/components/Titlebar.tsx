import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Square, X } from "lucide-react";
import logo from "../assets/logo.png";

const hasTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

type ResizeDir = "East" | "West" | "South" | "SouthEast" | "SouthWest";

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

  const isControl = (t: EventTarget) =>
    (t as HTMLElement).closest?.(".titlebar-controls") != null;

  const drag = (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    if (isControl(e.target)) return;
    void win?.startDragging();
  };

  const toggleMax = (e: React.MouseEvent) => {
    if (isControl(e.target)) return;
    void win?.toggleMaximize();
  };

  const resize = (dir: ResizeDir) => () => void win?.startResizeDragging(dir);

  return (
    <>
      <div className="titlebar" onMouseDown={drag} onDoubleClick={toggleMax}>
        <div className="titlebar-brand">
          <img src={logo} alt="" />
          <span>LinuxThemer</span>
        </div>
        <div className="titlebar-controls">
          <button aria-label="Minimize" onClick={() => void win?.minimize()}>
            <Minus size={14} />
          </button>
          <button aria-label="Maximize" onClick={() => void win?.toggleMaximize()}>
            {maximized ? <Copy size={12} /> : <Square size={12} />}
          </button>
          <button className="close" aria-label="Close" onClick={() => void win?.close()}>
            <X size={14} />
          </button>
        </div>
      </div>

      <div className="rz rz-w" onMouseDown={resize("West")} />
      <div className="rz rz-e" onMouseDown={resize("East")} />
      <div className="rz rz-s" onMouseDown={resize("South")} />
      <div className="rz rz-sw" onMouseDown={resize("SouthWest")} />
      <div className="rz rz-se" onMouseDown={resize("SouthEast")} />
    </>
  );
}
