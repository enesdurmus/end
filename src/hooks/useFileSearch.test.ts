import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { act, renderHook } from "@testing-library/react";
import { useFileSearch } from "./useFileSearch";
import { runActions } from "../lib/actions";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

beforeEach(() => { invoke.mockReset(); invoke.mockResolvedValue([]); vi.useFakeTimers(); });
afterEach(() => vi.useRealTimers());

test("does not search outside files mode", () => {
  renderHook(() => useFileSearch(runActions, "root", "hello"));
  vi.advanceTimersByTime(500);
  expect(invoke).not.toHaveBeenCalled();
});

test("does not search for queries shorter than 2 chars", () => {
  renderHook(() => useFileSearch(runActions, "files", "a"));
  vi.advanceTimersByTime(500);
  expect(invoke).not.toHaveBeenCalled();
});

test("debounces: waits 200ms before hitting the backend once", () => {
  renderHook(() => useFileSearch(runActions, "files", "report"));
  vi.advanceTimersByTime(199);
  expect(invoke).not.toHaveBeenCalled();
  vi.advanceTimersByTime(1);
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(invoke).toHaveBeenCalledWith("search_files", { query: "report" });
});

test("maps backend files into results", async () => {
  invoke.mockResolvedValue([{ name: "notes.txt", path: "/x/notes.txt" }]);
  const { result } = renderHook(() => useFileSearch(runActions, "files", "notes"));
  // fires the debounce timer AND flushes the invoke promise + state update
  await act(async () => { await vi.runAllTimersAsync(); });
  expect(result.current).toHaveLength(1);
  expect(result.current[0]).toMatchObject({ id: "file:/x/notes.txt", title: "notes.txt" });
});
