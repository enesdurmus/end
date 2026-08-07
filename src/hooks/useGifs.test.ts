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

test("local results come before remote results", async () => {
  const { result } = renderHook(() => useGifs("gif", "cat"));
  act(() => result.current.load());
  await waitFor(() => expect(result.current.gifs.length).toBe(2));
  expect(result.current.gifs.map((g) => g.id)).toEqual(["/g/cat.gif", "42"]);
});

test("a local gif that does not match the query is filtered out", async () => {
  const { result } = renderHook(() => useGifs("gif", "zzz"));
  act(() => result.current.load());
  await waitFor(() => expect(result.current.gifs.some((g) => g.source === "remote")).toBe(true));
  expect(result.current.gifs.some((g) => g.id === "/g/cat.gif")).toBe(false);
});

// the whole point of the local library is that it survives a dead network
test("a remote failure keeps local results and surfaces the message", async () => {
  invoke.mockImplementation((cmd: string) =>
    cmd === "gif_library" ? Promise.resolve([local]) : Promise.reject("klipy'ye ulaşılamadı")
  );
  const { result } = renderHook(() => useGifs("gif", "cat"));
  act(() => result.current.load());
  await waitFor(() => expect(result.current.error).toBe("klipy'ye ulaşılamadı"));
  expect(result.current.gifs.map((g) => g.id)).toEqual(["/g/cat.gif"]);
});

test("no remote request is made outside gif mode", async () => {
  renderHook(() => useGifs("root", "cat"));
  await new Promise((r) => setTimeout(r, 400));
  expect(invoke).not.toHaveBeenCalledWith("gif_search", expect.anything());
});
