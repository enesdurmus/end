import { useCallback, useEffect, useMemo, useReducer, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SearchBar } from "./components/SearchBar";
import { ResultList } from "./components/ResultList";
import { ClipboardView } from "./components/ClipboardView";
import { TranslateView } from "./components/TranslateView";
import { GifList } from "./components/GifList";
import { StatusBar } from "./components/StatusBar";
import { SnippetManager } from "./components/SnippetManager";
import { Window } from "./components/ui/Window";
import { buildCommands } from "./commands";
import { runActions } from "./lib/actions";
import { buildResults, langToResult, translationToResult, gifToResult } from "./lib/results";
import { LANGUAGES, languageName } from "./lib/languages";
import { navReducer, initialNav } from "./lib/navigation";
import { useApps } from "./hooks/useApps";
import { useClipboard } from "./hooks/useClipboard";
import { useSnippets } from "./hooks/useSnippets";
import { useFileSearch } from "./hooks/useFileSearch";
import { useTranslate } from "./hooks/useTranslate";
import { useTranslateHistory } from "./hooks/useTranslateHistory";
import { useGifs } from "./hooks/useGifs";
import { useLauncherEvents } from "./hooks/useLauncherEvents";
import { useKeyboardNav } from "./hooks/useKeyboardNav";
import { Gif } from "./types";

const PLACEHOLDER = {
  clipboard: "Search clipboard history...",
  files: "Search files...",
  translate: "Type to translate...",
  gif: "Search GIFs...",
  root: "Search...",
} as const;

export default function App() {
  const inputRef = useRef<HTMLInputElement>(null);
  const [nav, dispatch] = useReducer(navReducer, initialNav);
  const { mode, query, selected, managing, picking, source, target } = nav;

  const apps = useApps(runActions);
  const snips = useSnippets(runActions, managing);
  const files = useFileSearch(runActions, mode, query);
  const { clips, load: loadClips } = useClipboard(runActions);
  const { history, load: loadHistory, clear: clearHistory } = useTranslateHistory(runActions);
  const { entry, loading, error } = useTranslate(mode, query, source, target);
  const {
    gifs: gifList,
    error: gifError,
    actionError: gifActionError,
    load: loadGifs,
    favorite: favoriteGif,
    pasteGif: pasteGifAction,
  } = useGifs(mode, query, runActions.favorite, runActions.pasteGif);

  // Neither window is destroyed on close, so a mount-time-only read goes stale as
  // soon as the user changes "Translate To" in Preferences. Re-read on mount and on
  // every entry into translate mode. Clobbering `target` is safe: ⌘P persists first.
  const refreshLangs = useCallback(() => {
    invoke<{ translate_target: string }>("get_preferences").then((p) =>
      dispatch({ type: "setLangs", source: "auto", target: p.translate_target })
    );
  }, []);
  useEffect(() => { refreshLangs(); }, [refreshLangs]);
  const enterTranslate = useCallback(() => {
    loadHistory();
    refreshLangs();
  }, [loadHistory, refreshLangs]);

  const pickTarget = useCallback((code: string) => {
    dispatch({ type: "setTarget", code });
    // target only — the engine choice belongs to the Preferences window
    invoke("set_translate_prefs", { target: code, provider: null }).catch((e) =>
      console.error(e)
    );
  }, []);

  const commands = useMemo(
    () => buildCommands(dispatch, loadClips, enterTranslate, loadGifs),
    [loadClips, enterTranslate, loadGifs]
  );
  const langs = useMemo(() => LANGUAGES.map((l) => langToResult(l, pickTarget)), [pickTarget]);
  const translation = useMemo(
    () => (entry ? [translationToResult(entry, runActions)] : []),
    [entry]
  );
  // useGifs owns the library/remote state transition; here we just move the
  // selection onto the new local row once it lands, so Enter right after
  // ⌘Enter can't paste a gif other than the one just favourited.
  const gifActions = useMemo(
    () => ({
      pasteGif: pasteGifAction,
      favorite: async (g: Gif) => {
        const ok = await favoriteGif(g);
        if (ok) dispatch({ type: "selectIndex", index: 0 });
      },
    }),
    [pasteGifAction, favoriteGif]
  );
  const gifResults = useMemo(
    () => gifList.map((g) => gifToResult(g, gifActions)),
    [gifList, gifActions]
  );
  const results = useMemo(
    () =>
      buildResults(
        mode,
        query,
        { commands, apps, snips, clips, files, langs, translation, history, gifs: gifResults },
        picking
      ),
    [mode, query, picking, commands, apps, snips, clips, files, langs, translation, history, gifResults]
  );

  useLauncherEvents(dispatch, loadClips);
  useKeyboardNav({
    dispatch, results, selected, mode, query, picking,
    detected: entry?.from ?? "en",
    onClearHistory: clearHistory,
    inputRef,
  });

  if (managing) return <SnippetManager onClose={() => dispatch({ type: "closeManage" })} />;

  const typing = mode === "translate" && !!query.trim();
  // nav's `source` stays "auto" so the next request keeps auto-detecting;
  // the status bar shows what the backend actually detected
  const displaySource = typing && entry ? entry.from : source;
  const status: { left: string; hints: [string, string][] } = picking
    ? { left: "Target language", hints: [["Select", "↵"], ["Cancel", "esc"]] }
    : mode === "translate"
      ? {
          left: `${languageName(displaySource)} → ${languageName(target)}`,
          hints: typing
            ? [["Copy", "↵"], ["Paste", "⌘↵"], ["Lang", "⌘P"], ["Swap", "⌘S"]]
            : [["Copy", "↵"], ["Paste", "⌘↵"], ["Lang", "⌘P"], ["Clear", "⌘⌫"]],
        }
      : mode === "gif"
        ? {
            // KLIPY's terms are unread (their docs block automated fetches), so
            // attribute by default — free API, courteous, costs nothing if required.
            // Only when a KLIPY row is actually on screen: a local-only list has
            // nothing to attribute. gifError is the search failing; gifActionError
            // is Enter/⌘Enter failing on a result — either keeps the window open.
            left: gifError || gifActionError ||
              (gifList.some((g) => g.source === "remote") ? "Powered by KLIPY" : ""),
            hints: [["Paste", "↵"], ["Save", "⌘↵"], ["Folder", "⌘O"], ["Back", "esc"]],
          }
        : { left: `${results.length} results`, hints: [["Open", "↵"], ["Close", "esc"]] };

  return (
    <Window variant="floating">
      <SearchBar
        inputRef={inputRef}
        value={query}
        badge={
          picking ? "Target Language"
            : mode === "clipboard" ? "Clipboard History"
            : mode === "files" ? "Search Files"
            : mode === "translate" ? "Translate"
            : mode === "gif" ? "GIFs"
            : undefined
        }
        placeholder={picking ? "Search languages..." : PLACEHOLDER[mode]}
        onChange={(q) => dispatch({ type: "setQuery", query: q })}
      />
      {picking ? (
        <ResultList results={results} selected={selected} />
      ) : typing ? (
        <TranslateView entry={entry} loading={loading} error={error} />
      ) : mode === "clipboard" || mode === "translate" ? (
        <ClipboardView results={results} selected={selected} />
      ) : mode === "gif" ? (
        <GifList results={results} selected={selected} />
      ) : (
        <ResultList results={results} selected={selected} />
      )}
      <StatusBar left={status.left} hints={status.hints} />
    </Window>
  );
}
