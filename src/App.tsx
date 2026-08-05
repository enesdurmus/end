import { useMemo, useReducer, useRef } from "react";
import { SearchBar } from "./components/SearchBar";
import { ResultList } from "./components/ResultList";
import { ClipboardView } from "./components/ClipboardView";
import { StatusBar } from "./components/StatusBar";
import { SnippetManager } from "./components/SnippetManager";
import { Window } from "./components/ui/Window";
import { buildCommands } from "./commands";
import { runActions } from "./lib/actions";
import { buildResults } from "./lib/results";
import { navReducer, initialNav } from "./lib/navigation";
import { useApps } from "./hooks/useApps";
import { useClipboard } from "./hooks/useClipboard";
import { useSnippets } from "./hooks/useSnippets";
import { useFileSearch } from "./hooks/useFileSearch";
import { useLauncherEvents } from "./hooks/useLauncherEvents";
import { useKeyboardNav } from "./hooks/useKeyboardNav";

const PLACEHOLDER = {
  clipboard: "Search clipboard history...",
  files: "Search files...",
  root: "Search...",
} as const;

export default function App() {
  const inputRef = useRef<HTMLInputElement>(null);
  const [{ mode, query, selected, managing }, dispatch] = useReducer(navReducer, initialNav);

  const apps = useApps(runActions);
  const snips = useSnippets(runActions, managing);
  const files = useFileSearch(runActions, mode, query);
  const { clips, load: loadClips } = useClipboard(runActions);

  const commands = useMemo(() => buildCommands(dispatch, loadClips), [loadClips]);
  const results = useMemo(
    () => buildResults(mode, query, { commands, apps, snips, clips, files }),
    [mode, query, commands, apps, snips, clips, files]
  );

  useLauncherEvents(dispatch, loadClips);
  useKeyboardNav({ dispatch, results, selected, mode, query, inputRef });

  if (managing) return <SnippetManager onClose={() => dispatch({ type: "closeManage" })} />;

  return (
    <Window variant="floating">
      <SearchBar
        inputRef={inputRef}
        value={query}
        badge={mode === "clipboard" ? "Clipboard History" : mode === "files" ? "Search Files" : undefined}
        placeholder={PLACEHOLDER[mode]}
        onChange={(q) => dispatch({ type: "setQuery", query: q })}
      />
      {mode === "clipboard" ? (
        <ClipboardView results={results} selected={selected} />
      ) : (
        <ResultList results={results} selected={selected} />
      )}
      <StatusBar left={`${results.length} results`} hints={[["Open", "↵"], ["Close", "esc"]]} />
    </Window>
  );
}
