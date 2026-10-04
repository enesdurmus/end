import { expect, test } from "vitest";
import { navReducer, initialNav, NavState } from "./navigation";

test("setQuery updates query and resets selection to top", () => {
  const s: NavState = { ...initialNav, selected: 5 };
  expect(navReducer(s, { type: "setQuery", query: "saf" })).toMatchObject({ query: "saf", selected: 0 });
});

test("move clamps to [0, max-1]", () => {
  const s = { ...initialNav, selected: 0 };
  expect(navReducer(s, { type: "move", delta: -1, max: 3 }).selected).toBe(0); // no underflow
  expect(navReducer({ ...s, selected: 2 }, { type: "move", delta: 1, max: 3 }).selected).toBe(2); // no overflow
  expect(navReducer(s, { type: "move", delta: 1, max: 3 }).selected).toBe(1);
});

test("move on empty list stays at 0", () => {
  expect(navReducer(initialNav, { type: "move", delta: 1, max: 0 }).selected).toBe(0);
});

test("goMode enters a mode, clears query, leaves the form screens", () => {
  const s: NavState = { ...initialNav, mode: "root", query: "x", selected: 4, screen: "snippets" };
  expect(navReducer(s, { type: "goMode", mode: "clipboard" })).toMatchObject({
    mode: "clipboard", query: "", selected: 0, screen: "launcher",
  });
});

test("goRoot resets to root", () => {
  const s: NavState = { ...initialNav, mode: "files", query: "x", selected: 2, screen: "launcher" };
  expect(navReducer(s, { type: "goRoot" })).toMatchObject({ mode: "root", query: "", selected: 0 });
});

test("manage opens the snippet manager and goRoot leaves it", () => {
  const opened = navReducer(initialNav, { type: "manage" });
  expect(opened).toMatchObject({ screen: "snippets", query: "" });
  expect(navReducer(opened, { type: "goRoot" }).screen).toBe("launcher");
});

test("setLangs seeds the pair from persisted preferences", () => {
  expect(navReducer(initialNav, { type: "setLangs", source: "auto", target: "de" })).toMatchObject({
    source: "auto", target: "de",
  });
});

test("pickLang stashes the typed text and clears the query", () => {
  const s: NavState = { ...initialNav, mode: "translate", query: "merhaba", selected: 3 };
  expect(navReducer(s, { type: "pickLang" })).toMatchObject({
    picking: true, query: "", savedQuery: "merhaba", selected: 0,
  });
});

test("setTarget applies the language and restores the typed text", () => {
  const picking = navReducer(
    { ...initialNav, mode: "translate", query: "merhaba" },
    { type: "pickLang" }
  );
  expect(navReducer(picking, { type: "setTarget", code: "de" })).toMatchObject({
    picking: false, target: "de", query: "merhaba", savedQuery: "", selected: 0,
  });
});

test("cancelPick restores the typed text without changing the target", () => {
  const picking = navReducer(
    { ...initialNav, mode: "translate", query: "merhaba", target: "en" },
    { type: "pickLang" }
  );
  expect(navReducer(picking, { type: "cancelPick" })).toMatchObject({
    picking: false, target: "en", query: "merhaba",
  });
});

test("swap resolves auto to the detected language", () => {
  const s: NavState = { ...initialNav, mode: "translate", source: "auto", target: "en" };
  expect(navReducer(s, { type: "swap", detected: "tr" })).toMatchObject({ source: "en", target: "tr" });
});

test("swap flips an explicit pair", () => {
  const s: NavState = { ...initialNav, mode: "translate", source: "en", target: "tr" };
  expect(navReducer(s, { type: "swap", detected: "en" })).toMatchObject({ source: "tr", target: "en" });
});

test("goRoot leaves the picker and keeps the language pair", () => {
  const s: NavState = { ...initialNav, mode: "translate", picking: true, target: "de" };
  expect(navReducer(s, { type: "goRoot" })).toMatchObject({ mode: "root", picking: false, target: "de" });
});

test("leaving a mode clears the picker's stashed query", () => {
  const picking = navReducer(
    { ...initialNav, mode: "translate", query: "merhaba" },
    { type: "pickLang" }
  );
  expect(navReducer(picking, { type: "goRoot" })).toMatchObject({ query: "", savedQuery: "" });
  expect(navReducer(picking, { type: "goMode", mode: "clipboard" })).toMatchObject({
    query: "", savedQuery: "",
  });
});

// used after favouriting a gif, so the selection follows the new local row
// rather than whatever now sits at the old index
test("selectIndex sets the selection directly", () => {
  const s: NavState = { ...initialNav, mode: "gif", selected: 3 };
  expect(navReducer(s, { type: "selectIndex", index: 0 }).selected).toBe(0);
});

test("openSettings shows the preferences screen; goRoot leaves it", () => {
  const s = navReducer({ ...initialNav, picking: true, query: "x" }, { type: "openSettings" });
  expect(s).toMatchObject({ screen: "settings", picking: false, query: "" });
  expect(navReducer(s, { type: "goRoot" }).screen).toBe("launcher");
});
