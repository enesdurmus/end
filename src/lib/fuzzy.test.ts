import { expect, test } from "vitest";
import { fuzzyScore, fuzzyFilter } from "./fuzzy";

test("empty query gives every item equal positive score", () => {
  expect(fuzzyScore("", "Safari")).toBeGreaterThan(0);
});

test("non-matching character returns 0", () => {
  expect(fuzzyScore("xyz", "Safari")).toBe(0);
});

test("in-order subsequence matches", () => {
  expect(fuzzyScore("saf", "Safari")).toBeGreaterThan(0);
  expect(fuzzyScore("sfr", "Safari")).toBeGreaterThan(0);
});

test("prefix match scores higher than mid-string match", () => {
  expect(fuzzyScore("ter", "Terminal")).toBeGreaterThan(fuzzyScore("ter", "iTerm"));
});

test("filter sorts by score and drops zeros", () => {
  const out = fuzzyFilter(" term", ["Terminal", "iTerm", "Safari"], (s) => s);
  expect(out).toEqual(["Terminal", "iTerm"]);
});
