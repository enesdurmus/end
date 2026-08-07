import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Gif } from "../types";
import { Mode } from "../lib/navigation";
import { fuzzyFilter } from "../lib/fuzzy";

// ponytail: same shape as useTranslate — debounce + cancel flag, no request library
export function useGifs(mode: Mode, query: string) {
  const [library, setLibrary] = useState<Gif[]>([]);
  const [remote, setRemote] = useState<Gif[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const load = useCallback(() => {
    invoke<Gif[]>("gif_library").then(setLibrary).catch(() => setLibrary([]));
  }, []);

  const text = mode === "gif" ? query.trim() : "";

  useEffect(() => {
    if (!text) { setRemote([]); setError(""); setLoading(false); return; }
    let cancelled = false;
    setLoading(true);
    const t = setTimeout(() => {
      invoke<Gif[]>("gif_search", { query: text })
        .then((r) => { if (!cancelled) { setRemote(r); setError(""); } })
        .catch((e) => { if (!cancelled) { setRemote([]); setError(String(e)); } })
        .finally(() => { if (!cancelled) setLoading(false); });
    }, 250);
    return () => { cancelled = true; clearTimeout(t); };
  }, [text]);

  // Local first, deliberately: the library is small and hand-picked, so if
  // something in it matched what you typed, it is what you meant.
  const local = text ? fuzzyFilter(text, library, (g) => g.title) : library;
  const gifs = [...local, ...remote];

  return { gifs, error, loading, load };
}
