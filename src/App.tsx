import { useCallback, useEffect, useMemo, useReducer, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SearchBar } from "./components/SearchBar";
import { ResultList } from "./components/ResultList";
import { ClipboardView } from "./components/ClipboardView";
import { TranslateView } from "./components/TranslateView";
import { GifList } from "./components/GifList";
import { Tabs, Tab, TABS } from "./components/Tabs";
import { LangBar } from "./components/LangBar";
import { Detail } from "./components/Detail";
import { StatusBar } from "./components/StatusBar";
import { Preferences } from "./components/Preferences";
import { SnippetManager } from "./components/SnippetManager";
import { Window } from "./components/ui/Window";
import { Split } from "./components/ui/Split";
import { buildCommands } from "./commands";
import { runActions } from "./lib/actions";
import { buildResults, langToResult, translationToResult, alternativeToResult, gifToResult } from "./lib/results";
import { LANGUAGES, languageName } from "./lib/languages";
import { navReducer, initialNav } from "./lib/navigation";
import { checkForUpdates } from "./lib/updater";
import { useApps } from "./hooks/useApps";
import { useClipboard } from "./hooks/useClipboard";
import { useSnippets } from "./hooks/useSnippets";
import { useFileSearch } from "./hooks/useFileSearch";
import { useTranslate } from "./hooks/useTranslate";
import { useTranslateHistory } from "./hooks/useTranslateHistory";
import { useGifs } from "./hooks/useGifs";
import { useLauncherEvents } from "./hooks/useLauncherEvents";
import { useKeyboardNav } from "./hooks/useKeyboardNav";
import { Gif, Result } from "./types";

const TAB_TYPE: Partial<Record<Tab, Result["type"]>> = { Apps: "app", Commands: "command", Snippets: "snippet" };

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
  const { mode, query, selected, screen, picking, source, target } = nav;

  const apps = useApps(runActions);
  const snips = useSnippets(runActions, screen === "snippets");
  const files = useFileSearch(runActions, mode, query);
  const { clips, load: loadClips } = useClipboard(runActions);
  const { history, load: loadHistory, clear: clearHistory } = useTranslateHistory(runActions);
  const { entry, alternatives, meanings, loading, error } = useTranslate(mode, query, source, target);
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
  useEffect(() => { checkForUpdates().catch((e) => console.error("update check failed:", e)); }, []);
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

  // like pickTarget, but for the dropdown: it must not touch the query the picker borrows
  const changeTarget = (code: string) => {
    dispatch({ type: "setLangs", source, target: code });
    invoke("set_translate_prefs", { target: code, provider: null }).catch((e) => console.error(e));
  };

  const commands = useMemo(
    () => buildCommands(dispatch, loadClips, enterTranslate, loadGifs),
    [loadClips, enterTranslate, loadGifs]
  );
  const langs = useMemo(() => LANGUAGES.map((l) => langToResult(l, pickTarget)), [pickTarget]);
  const translation = useMemo(
    () => entry
      ? [translationToResult(entry, runActions), ...alternatives.map((a, i) => alternativeToResult(entry, a, i, runActions))]
      : [],
    [entry, alternatives]
  );
  // useGifs owns the library/remote state transition; here we just move the
  // selection onto the new local row once it lands, so Enter right after
  // ⌘Enter can't paste a gif other than the one just favourited. The query may
  // re-sort the merged list by score (see fuzzyFilter), so "index 0" is not a
  // safe assumption — remember the favourited gif's *id* instead, and resolve
  // it to a position once gifResults reflects the save (see the effect below).
  const pendingSelectId = useRef<string | null>(null);
  const gifActions = useMemo(
    () => ({
      pasteGif: pasteGifAction,
      favorite: async (g: Gif) => {
        const saved = await favoriteGif(g);
        if (saved) pendingSelectId.current = saved.id;
      },
    }),
    [pasteGifAction, favoriteGif]
  );
  const gifResults = useMemo(
    () => gifList.map((g) => gifToResult(g, gifActions)),
    [gifList, gifActions]
  );
  // Runs once per gifResults change (i.e. once the favourite's state update has
  // landed). If the id isn't found yet — or ever, e.g. it got filtered out by a
  // query change in flight — the selection is simply left alone rather than
  // guessed at; selecting a *different* gif is the one unacceptable outcome.
  // `selected` is shared across every mode (see navigation.ts), so this must
  // never fire once the user has left gif mode — otherwise a pending id that
  // resolves late moves the cursor in whatever list is now showing.
  useEffect(() => {
    if (mode !== "gif") { pendingSelectId.current = null; return; }
    const id = pendingSelectId.current;
    if (!id) return;
    const idx = gifResults.findIndex((r) => r.id === "gif:" + id);
    if (idx !== -1) {
      dispatch({ type: "selectIndex", index: idx });
      pendingSelectId.current = null;
    }
  }, [mode, gifResults]);
  const [tab, setTab] = useState<Tab>("All");
  const all = useMemo(
    () =>
      buildResults(
        mode,
        query,
        { commands, apps, snips, clips, files, langs, translation, history, gifs: gifResults },
        picking
      ),
    [mode, query, picking, commands, apps, snips, clips, files, langs, translation, history, gifResults]
  );
  // Apps/Commands/Snippets filter the root list; Files/Clipboard are real modes.
  // The tab bar stays up in those three screens so arrows can walk across it.
  const showTabs = !picking && (mode === "root" || mode === "files" || mode === "clipboard");
  const activeTab: Tab = mode === "files" ? "Files" : mode === "clipboard" ? "Clipboard" : tab;
  const results = useMemo(() => {
    const type = TAB_TYPE[tab];
    return mode === "root" && !picking && type ? all.filter((r) => r.type === type) : all;
  }, [all, mode, picking, tab]);
  const pickTab = (t: Tab) => {
    if (t === activeTab) return;
    const cmd = t === "Files" ? "cmd:files" : t === "Clipboard" ? "cmd:clipboard" : null;
    if (cmd) { commands.find((c) => c.id === cmd)?.run(); return; }
    if (mode !== "root") dispatch({ type: "goRoot" });
    setTab(t);
  };
  const onSelect = (index: number) => dispatch({ type: "selectIndex", index });
  const stepTab = (delta: number) => {
    const next = TABS[TABS.indexOf(activeTab) + delta];
    if (next) pickTab(next);
  };

  useLauncherEvents(dispatch, loadClips);
  const typing = mode === "translate" && !!query.trim();
  useKeyboardNav({
    dispatch, results, selected, mode, query, picking,
    detected: entry?.from ?? "en",
    onClearHistory: clearHistory,
    inputRef,
    rowShortcuts: picking || !(mode === "gif" || typing),
    overlay: screen !== "launcher",
    onTab: showTabs ? stepTab : null,
  });

  const close = () => dispatch({ type: "goRoot" });
  if (screen === "snippets") return <SnippetManager />;
  if (screen === "settings") return <Preferences onClose={close} />;

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
        : { left: `${results.length} results`, hints: [["Navigate", "↑↓"], ["Tabs", "←→"], ["Open", "↵"], ["Close", "esc"]] };

  return (
    <Window>
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
      {mode === "translate" && !picking && (
        <LangBar
          source={source}
          target={target}
          detected={typing && entry ? entry.from : undefined}
          onSource={(code) => dispatch({ type: "setLangs", source: code, target })}
          onTarget={changeTarget}
          onSwap={() => dispatch({ type: "swap", detected: entry?.from ?? "en" })}
        />
      )}
      {showTabs && <Tabs active={activeTab} onPick={pickTab} onSettings={() => dispatch({ type: "openSettings" })} />}
      {picking ? (
        <div className="flex flex-col flex-1 min-h-0 px-4 pb-3">
          <ResultList results={results} selected={selected} onSelect={onSelect} />
        </div>
      ) : typing ? (
        <TranslateView results={results} selected={selected} onSelect={onSelect} entry={entry} meanings={meanings} loading={loading} error={error} />
      ) : mode === "clipboard" || mode === "translate" ? (
        <ClipboardView results={results} selected={selected} onSelect={onSelect} />
      ) : mode === "gif" ? (
        <div className="flex flex-col flex-1 min-h-0 px-4 pb-3">
          <GifList results={results} selected={selected} onSelect={onSelect} />
        </div>
      ) : (
        <Split
          left={<ResultList results={results} selected={selected} onSelect={onSelect} />}
          right={<Detail result={results[selected]} />}
        />
      )}
      <StatusBar left={status.left} hints={status.hints} />
    </Window>
  );
}
