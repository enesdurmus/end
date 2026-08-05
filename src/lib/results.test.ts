import { expect, test, vi } from "vitest";
import { appToResult, clipToResult, snipToResult, fileToResult, buildResults,
         historyToResult, translationToResult, langToResult } from "./results";
import { RunActions } from "./actions";
import { Result, TranslationEntry } from "../types";

const fakeActions = (): RunActions => ({ paste: vi.fn(), open: vi.fn(), copy: vi.fn(), record: vi.fn() });

test("appToResult maps fields and run() opens the path", () => {
  const a = fakeActions();
  const r = appToResult({ name: "Safari", path: "/A/Safari.app" }, a);
  expect(r).toMatchObject({ id: "app:/A/Safari.app", type: "app", title: "Safari", subtitle: "/A/Safari.app" });
  r.run();
  expect(a.open).toHaveBeenCalledWith("/A/Safari.app");
});

test("fileToResult run() opens the path", () => {
  const a = fakeActions();
  fileToResult({ name: "notes.txt", path: "/x/notes.txt" }, a).run();
  expect(a.open).toHaveBeenCalledWith("/x/notes.txt");
});

test("clipToResult collapses whitespace in title, keeps full body, pastes on run", () => {
  const a = fakeActions();
  const r = clipToResult("hello   \n  world", 2, a);
  expect(r).toMatchObject({ id: "clip:2", title: "hello world", body: "hello   \n  world" });
  r.run();
  expect(a.paste).toHaveBeenCalledWith("hello   \n  world");
});

test("snipToResult falls back to text when keyword empty", () => {
  const a = fakeActions();
  expect(snipToResult({ keyword: "", text: "sig block" }, 0, a).title).toBe("sig block");
  expect(snipToResult({ keyword: "sig", text: "sig block" }, 0, a).title).toBe("sig");
});

// helpers to build Result stubs for buildResults
const r = (title: string, extra: Partial<Result> = {}): Result =>
  ({ id: title, type: "app", title, run: () => {}, ...extra });

const entry: TranslationEntry = { source: "merhaba", translated: "hello", from: "tr", to: "en" };

const emptyData = { commands: [], apps: [], snips: [], clips: [], files: [], langs: [], translation: [], history: [] };

test("buildResults clipboard mode: no query returns clips as-is, query fuzzy-filters", () => {
  const clips = [r("apple"), r("banana")];
  const data = { ...emptyData, clips };
  expect(buildResults("clipboard", "", data, false)).toEqual(clips);
  expect(buildResults("clipboard", "ban", data, false).map((x) => x.title)).toEqual(["banana"]);
});

test("buildResults files mode returns files verbatim", () => {
  const files = [r("a"), r("b")];
  expect(buildResults("files", "ignored", { ...emptyData, files }, false)).toEqual(files);
});

test("buildResults root mode: commands, then apps, then snippets (snippets only when query)", () => {
  const commands = [r("Clipboard History", { aliases: ["clips"] })];
  const apps = [r("Calendar")];
  const snips = [r("clip snippet")];
  const data = { ...emptyData, commands, apps, snips };
  // empty query -> no snippets, commands+apps in order
  expect(buildResults("root", "", data, false).map((x) => x.title)).toEqual(["Clipboard History", "Calendar"]);
  // "clip" matches the command (via alias) and the snippet, not Calendar
  const titles = buildResults("root", "clip", data, false).map((x) => x.title);
  expect(titles).toContain("Clipboard History");
  expect(titles).toContain("clip snippet");
  expect(titles).not.toContain("Calendar");
});

test("historyToResult copies on Enter and pastes on ⌘Enter", () => {
  const a = fakeActions();
  const r = historyToResult(entry, 0, a);
  expect(r).toMatchObject({ id: "trh:0", type: "translation", title: "hello", body: "merhaba" });
  r.run();
  expect(a.copy).toHaveBeenCalledWith("hello");
  r.altRun!();
  expect(a.paste).toHaveBeenCalledWith("hello");
});

test("historyToResult does not re-record an entry that is already in history", () => {
  const a = fakeActions();
  historyToResult(entry, 0, a).run();
  expect(a.record).not.toHaveBeenCalled();
});

test("translationToResult records the entry when it is used", () => {
  const a = fakeActions();
  const r = translationToResult(entry, a);
  r.run();
  expect(a.copy).toHaveBeenCalledWith("hello");
  expect(a.record).toHaveBeenCalledWith(entry);
});

test("langToResult reports the picked code", () => {
  const onPick = vi.fn();
  const r = langToResult({ code: "de", name: "German" }, onPick);
  expect(r).toMatchObject({ id: "lang:de", type: "language", title: "German", subtitle: "de" });
  r.run();
  expect(onPick).toHaveBeenCalledWith("de");
});

test("buildResults shows languages while picking, filtered by the query", () => {
  const langs = [
    langToResult({ code: "ja", name: "Japanese" }, vi.fn()),
    langToResult({ code: "de", name: "German" }, vi.fn()),
  ];
  const out = buildResults("translate", "jap", { ...emptyData, langs }, true);
  expect(out.map((r) => r.id)).toEqual(["lang:ja"]);
});

test("buildResults shows history on a blank translate query", () => {
  const history = [historyToResult(entry, 0, fakeActions())];
  expect(buildResults("translate", "   ", { ...emptyData, history }, false)).toEqual(history);
});

test("buildResults shows the live translation once the user types", () => {
  const translation = [translationToResult(entry, fakeActions())];
  expect(buildResults("translate", "merhaba", { ...emptyData, translation, history: [] }, false)).toEqual(translation);
});
