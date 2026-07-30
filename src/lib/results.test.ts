import { expect, test, vi } from "vitest";
import { appToResult, clipToResult, snipToResult, fileToResult, buildResults } from "./results";
import { RunActions } from "./actions";
import { Result } from "../types";

const fakeActions = (): RunActions => ({ paste: vi.fn(), open: vi.fn() });

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

test("buildResults clipboard mode: no query returns clips as-is, query fuzzy-filters", () => {
  const clips = [r("apple"), r("banana")];
  const data = { commands: [], apps: [], snips: [], clips, files: [] };
  expect(buildResults("clipboard", "", data)).toEqual(clips);
  expect(buildResults("clipboard", "ban", data).map((x) => x.title)).toEqual(["banana"]);
});

test("buildResults files mode returns files verbatim", () => {
  const files = [r("a"), r("b")];
  expect(buildResults("files", "ignored", { commands: [], apps: [], snips: [], clips: [], files })).toEqual(files);
});

test("buildResults root mode: commands, then apps, then snippets (snippets only when query)", () => {
  const commands = [r("Clipboard History", { aliases: ["clips"] })];
  const apps = [r("Calendar")];
  const snips = [r("clip snippet")];
  const data = { commands, apps, snips, clips: [], files: [] };
  // empty query -> no snippets, commands+apps in order
  expect(buildResults("root", "", data).map((x) => x.title)).toEqual(["Clipboard History", "Calendar"]);
  // "clip" matches the command (via alias) and the snippet, not Calendar
  const titles = buildResults("root", "clip", data).map((x) => x.title);
  expect(titles).toContain("Clipboard History");
  expect(titles).toContain("clip snippet");
  expect(titles).not.toContain("Calendar");
});
