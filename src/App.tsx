import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { SearchBar } from "./components/SearchBar";
import { ResultList } from "./components/ResultList";
import { SnippetManager } from "./components/SnippetManager";
import { fuzzyFilter } from "./lib/fuzzy";
import { Result } from "./types";
import "./App.css";

export default function App() {
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const [managing, setManaging] = useState(false);
  const [apps, setApps] = useState<Result[]>([]);
  const [files, setFiles] = useState<Result[]>([]);
  const [clips, setClips] = useState<Result[]>([]);
  const [snips, setSnips] = useState<Result[]>([]);

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
    invoke<string[]>("clipboard_history").then((list) =>
      setClips(list.map((text, i) => ({
        id: "clip:" + i,
        type: "clipboard" as const,
        title: text.replace(/\s+/g, " ").slice(0, 80),
        run: async () => { getCurrentWindow().hide(); await invoke("paste_text", { text }); },
      })))
    );
  }, []);

  useEffect(() => {
    invoke<{ name: string; path: string }[]>("list_apps").then((list) =>
      setApps(list.map((a) => ({
        id: "app:" + a.path,
        type: "app" as const,
        title: a.name,
        subtitle: a.path,
        run: async () => { await invoke("open_path", { path: a.path }); getCurrentWindow().hide(); },
      })))
    );
  }, []);

  useEffect(() => {
    if (query.trim().length < 2) { setFiles([]); return; }
    let cancelled = false;
    invoke<{ name: string; path: string }[]>("search_files", { query }).then((list) => {
      if (cancelled) return;
      setFiles(list.map((f) => ({
        id: "file:" + f.path,
        type: "file" as const,
        title: f.name,
        subtitle: f.path,
        run: async () => { await invoke("open_path", { path: f.path }); getCurrentWindow().hide(); },
      })));
    });
    return () => { cancelled = true; };
  }, [query]);

  const results = useMemo(() => {
    const base = [...fuzzyFilter(query, apps, (r) => r.title), ...files];
    const c = query.trim() ? fuzzyFilter(query, clips, (r) => r.title) : [];
    const sn = query.trim() ? fuzzyFilter(query, snips, (r) => r.title) : [];
    return [...base, ...sn, ...c];
  }, [query, apps, files, clips, snips]);

  useEffect(() => setSelected(0), [query]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") getCurrentWindow().hide();
      else if (e.key === "ArrowDown") setSelected((s) => Math.min(s + 1, results.length - 1));
      else if (e.key === "ArrowUp") setSelected((s) => Math.max(s - 1, 0));
      else if (e.key === "Enter") results[selected]?.run();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [results, selected]);

  if (managing) return <SnippetManager onClose={() => setManaging(false)} />;

  return (
    <div className="app">
      <SearchBar
        value={query}
        onChange={(v) => {
          if (v.trim() === ">snippets") { setManaging(true); setQuery(""); }
          else setQuery(v);
        }}
      />
      <ResultList results={results} selected={selected} />
    </div>
  );
}
