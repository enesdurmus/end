import { useEffect, useMemo, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { SearchBar } from "./components/SearchBar";
import { ResultList } from "./components/ResultList";
import { fuzzyFilter } from "./lib/fuzzy";
import { Result } from "./types";
import "./App.css";

const DUMMY: Result[] = [
  { id: "1", type: "app", title: "Safari", run: () => {} },
  { id: "2", type: "app", title: "Terminal", run: () => {} },
  { id: "3", type: "app", title: "iTerm", run: () => {} },
];

export default function App() {
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState(0);
  const results = useMemo(() => fuzzyFilter(query, DUMMY, (r) => r.title), [query]);

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

  return (
    <div className="app">
      <SearchBar value={query} onChange={setQuery} />
      <ResultList results={results} selected={selected} />
    </div>
  );
}
