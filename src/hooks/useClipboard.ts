import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Clip, Result } from "../types";
import { RunActions } from "../lib/actions";
import { clipToResult } from "../lib/results";

export function useClipboard(actions: RunActions): { clips: Result[]; load: () => void } {
  const [clips, setClips] = useState<Result[]>([]);

  const load = useCallback(() => {
    invoke<Clip[]>("clipboard_history").then((list) =>
      setClips(list.map((clip, i) => clipToResult(clip, i, actions)))
    );
  }, [actions]);

  return { clips, load };
}
