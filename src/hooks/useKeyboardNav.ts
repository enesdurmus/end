import { Dispatch, RefObject, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Result } from "../types";
import { Mode, NavAction } from "../lib/navigation";

type Args = {
  dispatch: Dispatch<NavAction>;
  results: Result[];
  selected: number;
  mode: Mode;
  query: string;
  picking: boolean;
  detected: string;
  onClearHistory: () => void;
  inputRef: RefObject<HTMLInputElement | null>;
};

// Owns all global keyboard behavior: type-to-focus, arrow nav, Enter to run, Esc/Backspace to go back.
export function useKeyboardNav({
  dispatch, results, selected, mode, query, picking, detected, onClearHistory, inputRef,
}: Args) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // any printable key while the input isn't focused -> send it to the search box
      if (e.key.length === 1 && !e.metaKey && !e.ctrlKey && !e.altKey) inputRef.current?.focus();

      // translate-only shortcuts; advertised in the status bar
      if (mode === "translate" && e.metaKey) {
        if (e.key === "p" && !picking) { e.preventDefault(); dispatch({ type: "pickLang" }); return; }
        if (e.key === "s" && !picking) { e.preventDefault(); dispatch({ type: "swap", detected }); return; }
        if (e.key === "Backspace" && !query.trim() && !picking) {
          e.preventDefault();
          onClearHistory();
          return;
        }
      }

      if (e.key === "Escape") {
        if (picking) dispatch({ type: "cancelPick" });
        else if (mode === "root") getCurrentWindow().hide();
        else dispatch({ type: "goRoot" });
      } else if (e.key === "Backspace" && query === "" && mode !== "root") {
        if (picking) dispatch({ type: "cancelPick" });
        else dispatch({ type: "goRoot" });
      } else if (e.key === "ArrowDown") {
        dispatch({ type: "move", delta: 1, max: results.length });
      } else if (e.key === "ArrowUp") {
        dispatch({ type: "move", delta: -1, max: results.length });
      } else if (e.key === "Enter") {
        const r = results[selected];
        // ⌘Enter is "the other action" (paste); plain Enter is the primary one
        if (e.metaKey && r?.altRun) r.altRun();
        else r?.run();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [dispatch, results, selected, mode, query, picking, detected, onClearHistory, inputRef]);
}
