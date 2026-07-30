import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Result } from "../types";
import { RunActions } from "../lib/actions";
import { fileToResult, RawFile } from "../lib/results";
import { Mode } from "../lib/navigation";

// ponytail: mdfind is expensive (spawns a process per keystroke) -> debounce + min length
export function useFileSearch(actions: RunActions, mode: Mode, query: string): Result[] {
  const [files, setFiles] = useState<Result[]>([]);
  const fileQuery = mode === "files" ? query.trim() : "";

  useEffect(() => {
    if (mode !== "files" || fileQuery.length < 2) { setFiles([]); return; }
    let cancelled = false;
    const t = setTimeout(() => {
      invoke<RawFile[]>("search_files", { query: fileQuery }).then((list) => {
        if (cancelled) return;
        setFiles(list.map((f) => fileToResult(f, actions)));
      });
    }, 200);
    return () => { cancelled = true; clearTimeout(t); };
  }, [fileQuery, mode, actions]);

  return files;
}
