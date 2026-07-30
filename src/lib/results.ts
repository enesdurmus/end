import { Result } from "../types";
import { RunActions } from "./actions";
import { fuzzyFilter } from "./fuzzy";
import { Mode } from "./navigation";

// Raw shapes returned by the Rust backend.
export type RawApp = { name: string; path: string };
export type RawFile = { name: string; path: string };
export type RawSnippet = { keyword: string; text: string };

export function appToResult(a: RawApp, actions: RunActions): Result {
  return {
    id: "app:" + a.path,
    type: "app",
    title: a.name,
    subtitle: a.path,
    run: () => actions.open(a.path),
  };
}

export function fileToResult(f: RawFile, actions: RunActions): Result {
  return {
    id: "file:" + f.path,
    type: "file",
    title: f.name,
    subtitle: f.path,
    run: () => actions.open(f.path),
  };
}

export function clipToResult(text: string, i: number, actions: RunActions): Result {
  return {
    id: "clip:" + i,
    type: "clipboard",
    title: text.replace(/\s+/g, " ").slice(0, 80),
    body: text,
    run: () => actions.paste(text),
  };
}

export function snipToResult(s: RawSnippet, i: number, actions: RunActions): Result {
  return {
    id: "snip:" + i,
    type: "snippet",
    title: s.keyword || s.text.slice(0, 40),
    subtitle: s.text.slice(0, 60),
    run: () => actions.paste(s.text),
  };
}

export type ResultData = {
  commands: Result[];
  apps: Result[];
  snips: Result[];
  clips: Result[];
  files: Result[];
};

// Mode-based composition of the visible result list. Mirrors the original App logic exactly.
export function buildResults(mode: Mode, query: string, data: ResultData): Result[] {
  if (mode === "clipboard") {
    return query.trim() ? fuzzyFilter(query, data.clips, (r) => r.title) : data.clips;
  }
  if (mode === "files") return data.files;

  const cmds = fuzzyFilter(query, data.commands, (r) => [r.title, ...(r.aliases ?? [])].join(" "));
  const base = fuzzyFilter(query, data.apps, (r) => r.title);
  const sn = query.trim() ? fuzzyFilter(query, data.snips, (r) => r.title) : [];
  return [...cmds, ...base, ...sn];
}
