import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Gif } from "../types";
import { Mode } from "../lib/navigation";
import { fuzzyFilter } from "../lib/fuzzy";

type FavoriteFn = (gif: Gif) => Promise<Gif>;
type PasteFn = (gif: Gif) => Promise<void> | void;

// ponytail: same shape as useTranslate — debounce + cancel flag, no request library
export function useGifs(mode: Mode, query: string, doFavorite: FavoriteFn, doPaste: PasteFn) {
  const [library, setLibrary] = useState<Gif[]>([]);
  const [remote, setRemote] = useState<Gif[]>([]);
  const [error, setError] = useState("");
  // Separate from `error` (= the search failed): a 404 mid-paste, an
  // unwritable library folder. Same status-bar slot, different source.
  const [actionError, setActionError] = useState("");

  const load = useCallback(() => {
    invoke<Gif[]>("gif_library").then(setLibrary).catch(() => setLibrary([]));
  }, []);

  const text = mode === "gif" ? query.trim() : "";

  useEffect(() => {
    if (!text) { setRemote([]); setError(""); return; }
    let cancelled = false;
    // clear the previous query's error, else it shows while the next loads
    setError("");
    const t = setTimeout(() => {
      invoke<Gif[]>("gif_search", { query: text })
        .then((r) => { if (!cancelled) { setRemote(r); setError(""); } })
        .catch((e) => { if (!cancelled) { setRemote([]); setError(String(e)); } });
    }, 250);
    return () => { cancelled = true; clearTimeout(t); };
  }, [text]);

  // Swaps the favourited row for its local copy in place — a re-list would
  // leave the stale remote row behind and show the gif twice. Resolves to the
  // saved Gif because the merged list may re-sort and the caller needs its id.
  const favorite = useCallback((g: Gif) => {
    return doFavorite(g).then(
      (saved) => {
        setLibrary((prev) => [saved, ...prev]);
        setRemote((prev) => prev.filter((r) => r.id !== g.id));
        setActionError("");
        return saved;
      },
      (e) => { setActionError(String(e)); return false as const; }
    );
  }, [doFavorite]);

  const pasteGif = useCallback((g: Gif) => {
    return Promise.resolve(doPaste(g)).then(
      () => setActionError(""),
      (e) => { setActionError(String(e)); }
    );
  }, [doPaste]);

  // Local first: the library is small and hand-picked, so a match there is
  // what you meant. Memoised — App's pending-selection effect depends on it.
  const gifs = useMemo(() => {
    const local = text ? fuzzyFilter(text, library, (g) => g.title) : library;
    return [...local, ...remote];
  }, [text, library, remote]);

  return { gifs, error, actionError, load, favorite, pasteGif };
}
