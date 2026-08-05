import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { Dispatch } from "react";
import { NavAction } from "../lib/navigation";

export function useLauncherEvents(dispatch: Dispatch<NavAction>, loadClips: () => void) {
  useEffect(() => {
    const uns = [
      listen("focus-search", () => dispatch({ type: "goRoot" })),
      listen("clipboard-mode", () => { dispatch({ type: "goMode", mode: "clipboard" }); loadClips(); }),
    ];
    return () => { uns.forEach((u) => u.then((f) => f())); };
  }, [dispatch, loadClips]);
}
