import { Dispatch, RefObject, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
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
  onTab: ((delta: number) => void) | null; // null when no tab bar is showing
};

export function useKeyboardNav({
  dispatch, results, selected, mode, query, picking, detected, onClearHistory, inputRef, onTab,
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

      if (mode === "gif" && e.metaKey && e.key === "o") {
        e.preventDefault();
        invoke("open_gif_dir");
        return;
      }

      // ←/→ walk the tabs, but only once the caret has nothing left to move over,
      // so editing the query with the arrows still works
      if (onTab && (e.key === "ArrowLeft" || e.key === "ArrowRight") && !e.metaKey && !e.altKey) {
        const el = inputRef.current;
        const atEdge = !el || (e.key === "ArrowLeft"
          ? el.selectionStart === 0 && el.selectionEnd === 0
          : el.selectionStart === el.value.length && el.selectionEnd === el.value.length);
        if (atEdge) { e.preventDefault(); onTab(e.key === "ArrowLeft" ? -1 : 1); return; }
      }

      if (e.key === "Escape") {
        if (picking) dispatch({ type: "cancelPick" });
        else if (mode === "root") invoke("close_launcher");
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
  }, [dispatch, results, selected, mode, query, picking, detected, onClearHistory, inputRef, onTab]);
}
