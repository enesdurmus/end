import { expect, test, vi } from "vitest";
import { appToResult, clipToResult, snipToResult, fileToResult, buildResults,
         historyToResult, translationToResult, langToResult, gifToResult, formatBytes } from "./results";
import { RunActions } from "./actions";
import { Result, TranslationEntry, Gif } from "../types";

const fakeActions = (): RunActions => ({
  paste: vi.fn(), pasteClip: vi.fn(), open: vi.fn(), copy: vi.fn(), record: vi.fn(),
  pasteGif: vi.fn(), favorite: vi.fn(),
});

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

test("clipToResult collapses whitespace in a text title and pastes the entry on run", () => {
  const a = fakeActions();
  const clip = { kind: "text", text: "hello   \n  world" } as const;
  const r = clipToResult(clip, 2, a);
  expect(r).toMatchObject({ id: "clip:2", title: "hello world", clip });
  r.run();
  expect(a.pasteClip).toHaveBeenCalledWith(clip);
});

test("clipToResult titles an image by its name and opens it on altRun", () => {
  const a = fakeActions();
  const clip = { kind: "image", path: "/i/a.png", name: "logo.png", width: 1920, height: 1080, bytes: 2_500_000 } as const;
  const r = clipToResult(clip, 0, a);
  expect(r).toMatchObject({ title: "logo.png", subtitle: "1920\u00d71080 \u00b7 2.4 MB" });
  r.altRun!();
  expect(a.open).toHaveBeenCalledWith("/i/a.png");
});

test("clipToResult names files, and counts them when there is more than one", () => {
  const a = fakeActions();
  const one = clipToResult({ kind: "files", paths: ["/d/report.pdf"] }, 0, a);
  expect(one).toMatchObject({ title: "report.pdf", subtitle: "/d/report.pdf" });

  const many = clipToResult({ kind: "files", paths: ["/d/a.txt", "/d/b.txt"] }, 1, a);
  expect(many).toMatchObject({ title: "a.txt, b.txt", subtitle: "2 files" });
  many.altRun!();
  expect(a.open).toHaveBeenCalledWith("/d/a.txt");
});

test("formatBytes stays readable across magnitudes", () => {
  expect(formatBytes(512)).toBe("512 B");
  expect(formatBytes(2048)).toBe("2.0 KB");
  expect(formatBytes(2_500_000)).toBe("2.4 MB");
  expect(formatBytes(20_000_000)).toBe("19 MB");
});

test("snipToResult falls back to text when keyword empty", () => {
  const a = fakeActions();
  expect(snipToResult({ keyword: "", text: "sig block" }, 0, a).title).toBe("sig block");
  expect(snipToResult({ keyword: "sig", text: "sig block" }, 0, a).title).toBe("sig");
});

const r = (title: string, extra: Partial<Result> = {}): Result =>
  ({ id: title, type: "app", title, run: () => {}, ...extra });

const entry: TranslationEntry = { source: "merhaba", translated: "hello", from: "tr", to: "en" };

const emptyData = { commands: [], apps: [], snips: [], clips: [], files: [], langs: [], translation: [], history: [], gifs: [] };

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
  expect(buildResults("root", "", data, false).map((x) => x.title)).toEqual(["Clipboard History", "Calendar"]);
  const titles = buildResults("root", "clip", data, false).map((x) => x.title);
  expect(titles).toContain("Clipboard History");
  expect(titles).toContain("clip snippet");
  expect(titles).not.toContain("Calendar");
});

test("historyToResult copies on Enter and pastes on ⌘Enter", () => {
  const a = fakeActions();
  const r = historyToResult(entry, 0, a);
  expect(r).toMatchObject({ id: "trh:0", type: "translation", title: "merhaba", body: "hello" });
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

const local: Gif = {
  id: "/gifs/cat.gif", title: "cat",
  preview: "/gifs/cat.gif", url: "/gifs/cat.gif", source: "local",
};
const remote: Gif = {
  id: "42", title: "Surprised Pikachu",
  preview: "https://t/tiny.gif", url: "https://t/full.gif", source: "remote",
};

const gifActions = () => {
  const calls: string[] = [];
  return {
    calls,
    a: {
      pasteGif: (g: Gif) => { calls.push("paste:" + g.id); },
      favorite: (g: Gif) => { calls.push("fav:" + g.id); },
    },
  };
};

test("a local gif pastes on Enter and is labelled as local", () => {
  const { calls, a } = gifActions();
  const r = gifToResult(local, a);
  expect(r.subtitle).toBe("local");
  r.run();
  expect(calls).toEqual(["paste:/gifs/cat.gif"]);
});

test("a remote gif favourites on ⌘Enter", () => {
  const { calls, a } = gifActions();
  const r = gifToResult(remote, a);
  expect(r.subtitle).toBe("remote");
  r.altRun!();
  expect(calls).toEqual(["fav:42"]);
});

// a local gif is already saved, so the second action must not re-download it
test("a local gif has no favourite action", () => {
  const { a } = gifActions();
  expect(gifToResult(local, a).altRun).toBeUndefined();
});

test("gif mode shows local results before remote results", () => {
  const { a } = gifActions();
  const rows = buildResults(
    "gif",
    "cat",
    {
      commands: [], apps: [], snips: [], clips: [], files: [],
      langs: [], translation: [], history: [],
      gifs: [gifToResult(remote, a), gifToResult(local, a)],
    },
    false
  );
  // buildResults must not reorder — useGifs already merged local-first
  expect(rows.map((r) => r.id)).toEqual(["gif:42", "gif:/gifs/cat.gif"]);
});
