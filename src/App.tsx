import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { SearchBar } from "./components/SearchBar";
import { ResultList } from "./components/ResultList";
import { SnippetManager } from "./components/SnippetManager";
import { fuzzyFilter } from "./lib/fuzzy";
import { Result } from "./types";

export default function App() {
  const inputRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const [managing, setManaging] = useState(false);
  const [mode, setMode] = useState<"root" | "clipboard" | "files">("root");
  const [apps, setApps] = useState<Result[]>([]);
  const [files, setFiles] = useState<Result[]>([]);
  const [clips, setClips] = useState<Result[]>([]);
  const [snips, setSnips] = useState<Result[]>([]);

  const loadClips = () =>
    invoke<string[]>("clipboard_history").then((list) =>
      setClips(list.map((text, i) => ({
        id: "clip:" + i,
        type: "clipboard" as const,
        title: text.replace(/\s+/g, " ").slice(0, 80),
        body: text,
        run: async () => { getCurrentWindow().hide(); await invoke("paste_text", { text }); },
      })))
    );

  const enter = (m: "clipboard" | "files") => { setQuery(""); setMode(m); if (m === "clipboard") loadClips(); };

  // mode-switch commands, Raycast-style: fuzzy-searchable in root, Enter switches view
  const commands: Result[] = [
    { id: "cmd:clipboard", type: "command", title: "Clipboard History", subtitle: "Browse and paste clipboard history",
      aliases: ["clipboard", "clips"], run: () => enter("clipboard") },
    { id: "cmd:files", type: "command", title: "Search Files", subtitle: "Find files by name",
      aliases: ["files", "find"], run: () => enter("files") },
    { id: "cmd:snippets", type: "command", title: "Manage Snippets", subtitle: "Create and edit snippets",
      aliases: ["snippets"], run: () => { setQuery(""); setManaging(true); } },
  ];

  useEffect(() => {
    invoke<{ keyword: string; text: string }[]>("list_snippets").then((list) =>
      setSnips(list.map((s, i) => ({
        id: "snip:" + i,
        type: "snippet" as const,
        title: s.keyword || s.text.slice(0, 40),
        subtitle: s.text.slice(0, 60),
        run: async () => { getCurrentWindow().hide(); await invoke("paste_text", { text: s.text }); },
      })))
    );
  }, [managing]);

  useEffect(() => {
    invoke<{ name: string; path: string }[]>("list_apps").then((list) => {
      setApps(list.map((a) => ({
        id: "app:" + a.path,
        type: "app" as const,
        title: a.name,
        subtitle: a.path,
        run: async () => { await invoke("open_path", { path: a.path }); getCurrentWindow().hide(); },
      })));
      // load real icons lazily; patch each app in as it resolves
      list.forEach((a) =>
        invoke<string | null>("app_icon", { path: a.path }).then((icon) => {
          if (!icon) return;
          setApps((prev) => prev.map((r) => (r.id === "app:" + a.path ? { ...r, icon } : r)));
        })
      );
    });
  }, []);

  // in files mode the query IS the file query
  const fileQuery = mode === "files" ? query.trim() : "";

  // ponytail: mdfind is expensive (spawns a process per keystroke) → debounce + limit
  useEffect(() => {
    if (mode !== "files" || fileQuery.length < 2) { setFiles([]); return; }
    let cancelled = false;
    const t = setTimeout(() => {
      invoke<{ name: string; path: string }[]>("search_files", { query: fileQuery }).then((list) => {
        if (cancelled) return;
        setFiles(list.map((f) => ({
          id: "file:" + f.path,
          type: "file" as const,
          title: f.name,
          subtitle: f.path,
          run: async () => { await invoke("open_path", { path: f.path }); getCurrentWindow().hide(); },
        })));
      });
    }, 200);
    return () => { cancelled = true; clearTimeout(t); };
  }, [fileQuery, mode]);

  const results = useMemo(() => {
    if (mode === "clipboard") {
      return query.trim() ? fuzzyFilter(query, clips, (r) => r.title) : clips;
    }
    if (mode === "files") return files;
    const cmds = fuzzyFilter(query, commands, (r) => [r.title, ...(r.aliases ?? [])].join(" "));
    const base = fuzzyFilter(query, apps, (r) => r.title);
    const sn = query.trim() ? fuzzyFilter(query, snips, (r) => r.title) : [];
    return [...cmds, ...base, ...sn];
  }, [mode, query, files, clips, snips, apps]);

  useEffect(() => setSelected(0), [query]);

  useEffect(() => {
    const uns = [
      listen("focus-search", () => { setQuery(""); setManaging(false); setMode("root"); }),
      listen("clipboard-mode", () => { setQuery(""); setManaging(false); setMode("clipboard"); loadClips(); }),
    ];
    return () => { uns.forEach((u) => u.then((f) => f())); };
  }, []);

  useEffect(() => {
    const toRoot = () => { setMode("root"); setQuery(""); };
    const onKey = (e: KeyboardEvent) => {
      // any printable key while the input isn't focused -> send it to the search box
      if (e.key.length === 1 && !e.metaKey && !e.ctrlKey && !e.altKey) inputRef.current?.focus();
      if (e.key === "Escape") { if (mode === "root") getCurrentWindow().hide(); else toRoot(); }
      else if (e.key === "Backspace" && query === "" && mode !== "root") toRoot();
      else if (e.key === "ArrowDown") setSelected((s) => Math.min(s + 1, results.length - 1));
      else if (e.key === "ArrowUp") setSelected((s) => Math.max(s - 1, 0));
      else if (e.key === "Enter") results[selected]?.run();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [results, selected, mode, query]);

  if (managing) return <SnippetManager onClose={() => setManaging(false)} />;

  return (
    <div className="app">
      <SearchBar
        inputRef={inputRef}
        value={query}
        badge={mode === "clipboard" ? "Clipboard History" : mode === "files" ? "Search Files" : undefined}
        placeholder={mode === "clipboard" ? "Search clipboard history..." : mode === "files" ? "Search files..." : "Search..."}
        onChange={setQuery}
      />
      {mode === "clipboard" ? (
        <div className="split">
          <ResultList results={results} selected={selected} />
          <div className="preview">
            {results[selected]?.body ? (
              <pre className="preview-text">{results[selected].body!.slice(0, 5000)}</pre>
            ) : (
              <div className="preview-empty">No selection</div>
            )}
          </div>
        </div>
      ) : (
        <ResultList results={results} selected={selected} />
      )}
      <div className="footer">
        <span>{results.length} results</span>
        <span className="actions">
          <span>Open<span className="kbd">↵</span></span>
          <span>Close<span className="kbd">esc</span></span>
        </span>
      </div>
    </div>
  );
}
