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
  inputRef: RefObject<HTMLInputElement | null>;
};

// Owns all global keyboard behavior: type-to-focus, arrow nav, Enter to run, Esc/Backspace to go back.
export function useKeyboardNav({ dispatch, results, selected, mode, query, inputRef }: Args) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // any printable key while the input isn't focused -> send it to the search box
      if (e.key.length === 1 && !e.metaKey && !e.ctrlKey && !e.altKey) inputRef.current?.focus();

      if (e.key === "Escape") {
        if (mode === "root") getCurrentWindow().hide();
        else dispatch({ type: "goRoot" });
      } else if (e.key === "Backspace" && query === "" && mode !== "root") {
        dispatch({ type: "goRoot" });
      } else if (e.key === "ArrowDown") {
        dispatch({ type: "move", delta: 1, max: results.length });
      } else if (e.key === "ArrowUp") {
        dispatch({ type: "move", delta: -1, max: results.length });
      } else if (e.key === "Enter") {
        results[selected]?.run();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [dispatch, results, selected, mode, query, inputRef]);
}
