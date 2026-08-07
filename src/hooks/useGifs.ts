import { useCallback, useEffect, useState } from "react";
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
  // Separate from `error`: that one is "the search failed", this one is "the
  // action on a result failed" (a 404 mid-paste, an unwritable library folder).
  // Both render in the same status-bar slot, but they come from different places.
  const [actionError, setActionError] = useState("");

  const load = useCallback(() => {
    invoke<Gif[]>("gif_library").then(setLibrary).catch(() => setLibrary([]));
  }, []);

  const text = mode === "gif" ? query.trim() : "";

  useEffect(() => {
    if (!text) { setRemote([]); setError(""); return; }
    let cancelled = false;
    // clear a stale error from the previous query up front, mirroring
    // useTranslate — otherwise a resolved failure keeps showing while the
    // next search is already loading.
    setError("");
    const t = setTimeout(() => {
      invoke<Gif[]>("gif_search", { query: text })
        .then((r) => { if (!cancelled) { setRemote(r); setError(""); } })
        .catch((e) => { if (!cancelled) { setRemote([]); setError(String(e)); } });
    }, 250);
    return () => { cancelled = true; clearTimeout(t); };
  }, [text]);

  // Replaces the favourited row with its local copy in place, rather than
  // re-listing the folder: a re-list refreshes `library` but leaves the stale
  // row sitting in `remote`, so the same gif ends up appearing twice. Resolves
  // to whether it succeeded, so the caller can move the selection onto the new
  // local row (index 0) without risking it landing on an unrelated gif.
  const favorite = useCallback((g: Gif) => {
    return doFavorite(g).then(
      (saved) => {
        setLibrary((prev) => [saved, ...prev]);
        setRemote((prev) => prev.filter((r) => r.id !== g.id));
        setActionError("");
        return true;
      },
      (e) => { setActionError(String(e)); return false; }
    );
  }, [doFavorite]);

  const pasteGif = useCallback((g: Gif) => {
    return Promise.resolve(doPaste(g)).then(
      () => setActionError(""),
      (e) => { setActionError(String(e)); }
    );
  }, [doPaste]);

  // Local first, deliberately: the library is small and hand-picked, so if
  // something in it matched what you typed, it is what you meant.
  const local = text ? fuzzyFilter(text, library, (g) => g.title) : library;
  const gifs = [...local, ...remote];

  return { gifs, error, actionError, load, favorite, pasteGif };
}
