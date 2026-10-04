import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Dispatch } from "react";
import { NavAction } from "../lib/navigation";

// Timeout, not rAF: a hidden window gets no animation frames.
const showAfterPaint = () => setTimeout(() => invoke("show_launcher"), 0);

export function useLauncherEvents(dispatch: Dispatch<NavAction>, loadClips: () => void) {
  useEffect(() => {
    const uns = [
      listen("focus-search", () => { dispatch({ type: "goRoot" }); showAfterPaint(); }),
      listen("open-settings", () => { dispatch({ type: "openSettings" }); showAfterPaint(); }),
      listen("clipboard-mode", () => {
        dispatch({ type: "goMode", mode: "clipboard" });
        loadClips();
        showAfterPaint();
      }),
    ];
    return () => { uns.forEach((u) => u.then((f) => f())); };
  }, [dispatch, loadClips]);
}
