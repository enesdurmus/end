import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Result } from "../types";
import { RunActions } from "../lib/actions";
import { clipToResult } from "../lib/results";

// Clipboard history is loaded on demand (when entering clipboard mode), not on mount.
export function useClipboard(actions: RunActions): { clips: Result[]; load: () => void } {
  const [clips, setClips] = useState<Result[]>([]);

  const load = useCallback(() => {
    invoke<string[]>("clipboard_history").then((list) =>
      setClips(list.map((text, i) => clipToResult(text, i, actions)))
    );
  }, [actions]);

  return { clips, load };
}
