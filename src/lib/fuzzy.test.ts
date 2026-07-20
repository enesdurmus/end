import { expect, test } from "vitest";
import { fuzzyScore, fuzzyFilter } from "./fuzzy";

test("boş query her şeye eşit pozitif skor", () => {
  expect(fuzzyScore("", "Safari")).toBeGreaterThan(0);
});

test("eşleşmeyen karakter 0 döner", () => {
  expect(fuzzyScore("xyz", "Safari")).toBe(0);
});

test("sıralı subsequence eşleşir", () => {
  expect(fuzzyScore("saf", "Safari")).toBeGreaterThan(0);
  expect(fuzzyScore("sfr", "Safari")).toBeGreaterThan(0);
});

test("prefix eşleşme, ortadan eşleşmeden yüksek skor", () => {
  expect(fuzzyScore("ter", "Terminal")).toBeGreaterThan(fuzzyScore("ter", "iTerm"));
});

test("filter skora göre sıralar ve sıfırları eler", () => {
  const out = fuzzyFilter(" term", ["Terminal", "iTerm", "Safari"], (s) => s);
  expect(out).toEqual(["Terminal", "iTerm"]);
});
