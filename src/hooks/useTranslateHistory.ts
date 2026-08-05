import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Result, TranslationEntry } from "../types";
import { RunActions } from "../lib/actions";
import { historyToResult } from "../lib/results";

// Loaded on demand (when entering translate mode), like useClipboard.
export function useTranslateHistory(actions: RunActions) {
  const [history, setHistory] = useState<Result[]>([]);

  const load = useCallback(() => {
    invoke<TranslationEntry[]>("translate_history").then((list) =>
      setHistory(list.map((e, i) => historyToResult(e, i, actions)))
    );
  }, [actions]);

  const clear = useCallback(() => {
    invoke("clear_translate_history").then(() => setHistory([]));
  }, []);

  return { history, load, clear };
}
