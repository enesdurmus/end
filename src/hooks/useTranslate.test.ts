import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { act, renderHook } from "@testing-library/react";
import { useTranslate } from "./useTranslate";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

beforeEach(() => {
  invoke.mockReset();
  invoke.mockResolvedValue({ text: "hello", detected: "tr" });
  vi.useFakeTimers();
});
afterEach(() => vi.useRealTimers());

test("does not translate outside translate mode", () => {
  renderHook(() => useTranslate("root", "merhaba", "auto", "en"));
  vi.advanceTimersByTime(1000);
  expect(invoke).not.toHaveBeenCalled();
});

test("does not translate a blank query", () => {
  renderHook(() => useTranslate("translate", "   ", "auto", "en"));
  vi.advanceTimersByTime(1000);
  expect(invoke).not.toHaveBeenCalled();
});

test("debounces 300ms and sends the target language", () => {
  renderHook(() => useTranslate("translate", "merhaba", "auto", "de"));
  vi.advanceTimersByTime(299);
  expect(invoke).not.toHaveBeenCalled();
  vi.advanceTimersByTime(1);
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(invoke).toHaveBeenCalledWith("translate", { text: "merhaba", from: null, to: "de" });
});

test("sends an explicit source once the user pins one", () => {
  renderHook(() => useTranslate("translate", "hello", "en", "tr"));
  vi.advanceTimersByTime(300);
  expect(invoke).toHaveBeenCalledWith("translate", { text: "hello", from: "en", to: "tr" });
});

test("builds an entry from the query and the response", async () => {
  const { result } = renderHook(() => useTranslate("translate", "merhaba", "auto", "en"));
  await act(async () => { await vi.runAllTimersAsync(); });
  expect(result.current.entry).toEqual({
    source: "merhaba", translated: "hello", from: "tr", to: "en",
  });
  expect(result.current.error).toBe("");
  expect(result.current.loading).toBe(false);
});

test("surfaces a backend error and drops the stale entry", async () => {
  invoke.mockRejectedValue("network unreachable");
  const { result } = renderHook(() => useTranslate("translate", "merhaba", "auto", "en"));
  await act(async () => { await vi.runAllTimersAsync(); });
  expect(result.current.entry).toBeNull();
  expect(result.current.error).toBe("network unreachable");
});
