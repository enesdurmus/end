import { Gif, Result, TranslationEntry } from "../types";
import { RunActions } from "./actions";
import { fuzzyFilter } from "./fuzzy";
import { Mode } from "./navigation";
import { Language } from "./languages";

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

export function langToResult(l: Language, onPick: (code: string) => void): Result {
  return {
    id: "lang:" + l.code,
    type: "language",
    title: l.name,
    subtitle: l.code,
    run: () => onPick(l.code),
  };
}

// A history row: already stored, so using it must not re-record it.
export function historyToResult(e: TranslationEntry, i: number, actions: RunActions): Result {
  return {
    id: "trh:" + i,
    type: "translation",
    title: e.source.replace(/\s+/g, " ").slice(0, 80),
    subtitle: `${e.from} → ${e.to}`,
    body: e.translated,
    run: () => actions.copy(e.translated),
    altRun: () => actions.paste(e.translated),
  };
}

// The live translation, as a single synthetic result so Enter/⌘Enter reuse the
// same key handling as every other row. Using it is what writes it to history.
export function translationToResult(e: TranslationEntry, actions: RunActions): Result {
  return {
    id: "tr:live",
    type: "translation",
    title: e.translated,
    subtitle: `${e.from} → ${e.to}`,
    body: e.translated,
    run: () => { actions.record(e); return actions.copy(e.translated); },
    altRun: () => { actions.record(e); return actions.paste(e.translated); },
  };
}

// Enter pastes; ⌘Enter saves it to the library. A local gif is already saved, so
// it gets no second action rather than a no-op one.
export function gifToResult(g: Gif, actions: RunActions): Result {
  return {
    id: "gif:" + g.id,
    type: "gif",
    title: g.title,
    subtitle: g.source,
    icon: g.preview,
    run: () => actions.pasteGif(g),
    altRun: g.source === "remote" ? () => { actions.favorite(g); } : undefined,
  };
}

export type ResultData = {
  commands: Result[];
  apps: Result[];
  snips: Result[];
  clips: Result[];
  files: Result[];
  langs: Result[];
  translation: Result[]; // 0 or 1 entries — the live translation
  history: Result[];
  gifs: Result[];
};

export function buildResults(mode: Mode, query: string, data: ResultData, picking: boolean): Result[] {
  // the picker borrows the whole list, whatever mode we are in
  if (picking) return fuzzyFilter(query, data.langs, (r) => r.title);
  if (mode === "translate") return query.trim() ? data.translation : data.history;
  if (mode === "clipboard") {
    return query.trim() ? fuzzyFilter(query, data.clips, (r) => r.title) : data.clips;
  }
  if (mode === "files") return data.files;
  if (mode === "gif") return data.gifs;

  const cmds = fuzzyFilter(query, data.commands, (r) => [r.title, ...(r.aliases ?? [])].join(" "));
  const base = fuzzyFilter(query, data.apps, (r) => r.title);
  const sn = query.trim() ? fuzzyFilter(query, data.snips, (r) => r.title) : [];
  return [...cmds, ...base, ...sn];
}
