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

test("goMode enters a mode, clears query, exits managing", () => {
  const s: NavState = { mode: "root", query: "x", selected: 4, managing: true };
  expect(navReducer(s, { type: "goMode", mode: "clipboard" })).toMatchObject({
    mode: "clipboard", query: "", managing: false,
  });
});

test("goRoot resets to root", () => {
  const s: NavState = { mode: "files", query: "x", selected: 2, managing: false };
  expect(navReducer(s, { type: "goRoot" })).toMatchObject({ mode: "root", query: "" });
});

test("manage / closeManage toggle the snippet manager", () => {
  const opened = navReducer(initialNav, { type: "manage" });
  expect(opened).toMatchObject({ managing: true, query: "" });
  expect(navReducer(opened, { type: "closeManage" }).managing).toBe(false);
});
