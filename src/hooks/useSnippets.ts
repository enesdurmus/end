import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Result } from "../types";
import { RunActions } from "../lib/actions";
import { snipToResult, RawSnippet } from "../lib/results";

// Reloads whenever the snippet manager closes (managing flips), so edits show up in root.
export function useSnippets(actions: RunActions, managing: boolean): Result[] {
  const [snips, setSnips] = useState<Result[]>([]);

  useEffect(() => {
    invoke<RawSnippet[]>("list_snippets").then((list) =>
      setSnips(list.map((s, i) => snipToResult(s, i, actions)))
    );
  }, [managing, actions]);

  return snips;
}
