import { renderHook, waitFor, act } from "@testing-library/react";
import { vi, test, expect, beforeEach } from "vitest";
import { useGifs } from "./useGifs";
import { Gif } from "../types";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const local: Gif = { id: "/g/cat.gif", title: "cat", preview: "/g/cat.gif", url: "/g/cat.gif", source: "local" };
const remote: Gif = { id: "42", title: "cat dance", preview: "https://t/t.gif", url: "https://t/f.gif", source: "remote" };

beforeEach(() => {
  invoke.mockReset();
  invoke.mockImplementation((cmd: string) =>
    cmd === "gif_library" ? Promise.resolve([local]) : Promise.resolve([remote])
  );
});

const noop = () => {};

test("local results come before remote results", async () => {
  const { result } = renderHook(() => useGifs("gif", "cat", vi.fn(), noop));
  act(() => result.current.load());
  await waitFor(() => expect(result.current.gifs.length).toBe(2));
  expect(result.current.gifs.map((g) => g.id)).toEqual(["/g/cat.gif", "42"]);
});

test("a local gif that does not match the query is filtered out", async () => {
  const { result } = renderHook(() => useGifs("gif", "zzz", vi.fn(), noop));
  act(() => result.current.load());
  await waitFor(() => expect(result.current.gifs.some((g) => g.source === "remote")).toBe(true));
  expect(result.current.gifs.some((g) => g.id === "/g/cat.gif")).toBe(false);
});

// the whole point of the local library is that it survives a dead network
test("a remote failure keeps local results and surfaces the message", async () => {
  invoke.mockImplementation((cmd: string) =>
    cmd === "gif_library" ? Promise.resolve([local]) : Promise.reject("klipy'ye ulaşılamadı")
  );
  const { result } = renderHook(() => useGifs("gif", "cat", vi.fn(), noop));
  act(() => result.current.load());
  await waitFor(() => expect(result.current.error).toBe("klipy'ye ulaşılamadı"));
  expect(result.current.gifs.map((g) => g.id)).toEqual(["/g/cat.gif"]);
});

test("no remote request is made outside gif mode", async () => {
  renderHook(() => useGifs("root", "cat", vi.fn(), noop));
  await new Promise((r) => setTimeout(r, 400));
  expect(invoke).not.toHaveBeenCalledWith("gif_search", expect.anything());
});

// item 1: favouriting must not duplicate the row the remote gif came from
test("favorite replaces the remote row with the returned local copy instead of duplicating it", async () => {
  const savedLocal: Gif = { id: "/g/cat-dance.gif", title: "cat dance", preview: "/g/cat-dance.gif", url: "/g/cat-dance.gif", source: "local" };
  const doFavorite = vi.fn().mockResolvedValue(savedLocal);
  const { result } = renderHook(() => useGifs("gif", "cat", doFavorite, noop));
  act(() => result.current.load());
  await waitFor(() => expect(result.current.gifs.length).toBe(2));

  await act(async () => { await result.current.favorite(remote); });

  const ids = result.current.gifs.map((g) => g.id);
  expect(ids).toEqual(["/g/cat-dance.gif", "/g/cat.gif"]);
  expect(ids).not.toContain("42");
});

test("favorite resolves the saved gif on success so the caller can select it by id", async () => {
  const savedLocal: Gif = { ...local, id: "/g/new.gif" };
  const doFavorite = vi.fn().mockResolvedValue(savedLocal);
  const { result } = renderHook(() => useGifs("gif", "cat", doFavorite, noop));
  let ok: Gif | false | undefined;
  await act(async () => { ok = await result.current.favorite(remote); });
  expect(ok).toEqual(savedLocal);
});

// item 2: a failed action must surface an error instead of an unhandled rejection
test("a failed favorite sets actionError and does not touch the library", async () => {
  const doFavorite = vi.fn().mockRejectedValue("disk full");
  const { result } = renderHook(() => useGifs("gif", "cat", doFavorite, noop));
  act(() => result.current.load());
  await waitFor(() => expect(result.current.gifs.length).toBe(2));

  let ok: Gif | false | undefined;
  await act(async () => { ok = await result.current.favorite(remote); });

  expect(ok).toBe(false);
  expect(result.current.actionError).toBe("disk full");
  expect(result.current.gifs.map((g) => g.id)).toContain("42");
});

test("a failed paste sets actionError", async () => {
  const doPaste = vi.fn().mockRejectedValue("404");
  const { result } = renderHook(() => useGifs("gif", "cat", vi.fn(), doPaste));
  await act(async () => { await result.current.pasteGif(local); });
  expect(result.current.actionError).toBe("404");
});

// item 5: a stale search error must not survive into the next debounce cycle
test("a new query clears a previous search error before the next result lands", async () => {
  invoke.mockImplementation((cmd: string) =>
    cmd === "gif_library" ? Promise.resolve([local]) : Promise.reject("klipy'ye ulaşılamadı")
  );
  const { result, rerender } = renderHook(
    ({ q }: { q: string }) => useGifs("gif", q, vi.fn(), noop),
    { initialProps: { q: "cat" } }
  );
  await waitFor(() => expect(result.current.error).toBe("klipy'ye ulaşılamadı"));

  invoke.mockImplementation((cmd: string) =>
    cmd === "gif_library" ? Promise.resolve([local]) : new Promise(() => {})
  );
  rerender({ q: "dog" });
  // the debounce hasn't fired yet, but the stale error from "cat" must be gone already
  expect(result.current.error).toBe("");
});
