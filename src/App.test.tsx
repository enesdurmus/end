import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import App from "./App";
import { Gif } from "./types";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...a: unknown[]) => invoke(...a),
  convertFileSrc: (p: string) => p,
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));

const bandLocal: Gif = { id: "/g/band.gif", title: "band", preview: "/g/band.gif", url: "/g/band.gif", source: "local" };
const bannedRemote: Gif = { id: "42", title: "banned gif", preview: "https://t/t.gif", url: "https://t/f.gif", source: "remote" };
const bannedSavedLocal: Gif = { id: "/g/banned-gif.gif", title: "banned gif", preview: "/g/banned-gif.gif", url: "/g/banned-gif.gif", source: "local" };

beforeEach(() => {
  // jsdom has no layout engine, so this DOM method used by ResultList/GifList
  // to keep the selection in view is simply absent
  Element.prototype.scrollIntoView = vi.fn();
  invoke.mockReset();
  invoke.mockImplementation((cmd: string) => {
    switch (cmd) {
      case "get_preferences": return Promise.resolve({ translate_target: "en" });
      case "list_apps": return Promise.resolve([]);
      case "list_snippets": return Promise.resolve([]);
      case "gif_library": return Promise.resolve([bandLocal]);
      case "gif_search": return Promise.resolve([bannedRemote]);
      case "favorite_gif": return Promise.resolve(bannedSavedLocal);
      default: return Promise.resolve(undefined);
    }
  });
});

afterEach(() => { cleanup(); vi.restoreAllMocks(); });

function getInput(): HTMLInputElement {
  return document.querySelector("input") as HTMLInputElement;
}

// item 3: a scoring collision (the just-favourited row sorts behind an unrelated
// higher-scoring row for the current query) must not leave the selection on the
// wrong gif — reproduces the "band" / "banned gif" case from the design doc.
test("favouriting selects the gif that was favourited even when it re-sorts behind another row", async () => {
  render(<App />);

  // enter gif mode via the "Search GIFs" command
  fireEvent.change(getInput(), { target: { value: "gif" } });
  await waitFor(() => expect(screen.queryByText("Search GIFs")).not.toBeNull());
  fireEvent.keyDown(window, { key: "Enter" });
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("gif_library"));

  // query "band": the local library already has a gif titled "band" (higher
  // fuzzy score), and the remote search returns "banned gif" (lower score)
  fireEvent.change(getInput(), { target: { value: "band" } });
  await waitFor(() => expect(screen.queryByText("banned gif")).not.toBeNull(), { timeout: 2000 });
  await waitFor(() => expect(screen.queryByText("band")).not.toBeNull());

  // select the remote "banned gif" row (index 1: "band" sorts first) and favourite it
  fireEvent.keyDown(window, { key: "ArrowDown" });
  fireEvent.keyDown(window, { key: "Enter", metaKey: true });

  await waitFor(() => expect(invoke).toHaveBeenCalledWith("favorite_gif", { gif: bannedRemote }));

  // after the save, both "band" (score 25) and "banned gif" (score 21) are local,
  // and "band" still sorts first — selectIndex:0 would land on the wrong gif
  await waitFor(() => {
    const rows = Array.from(document.querySelectorAll("li > div"));
    const selected = rows.find((row) => row.className.split(/\s+/).includes("bg-sel"));
    expect(selected?.textContent).toContain("banned gif");
  });
});

// `selected` is a single index shared by every mode (see navigation.ts). If the
// pending-selection effect in App.tsx fires after the user has already left gif
// mode, it dispatches selectIndex against whatever list is showing now — moving
// the cursor onto an app/command the user never picked. Must not happen.
test("leaving gif mode before the favourite resolves does not move the selection in the new mode", async () => {
  // hold the favorite_gif response open so the pending id is still unresolved
  // when the mode switch happens
  let resolveFavorite!: (g: Gif) => void;
  invoke.mockImplementation((cmd: string) => {
    switch (cmd) {
      case "get_preferences": return Promise.resolve({ translate_target: "en" });
      case "list_apps": return Promise.resolve([]);
      case "list_snippets": return Promise.resolve([]);
      case "gif_library": return Promise.resolve([bandLocal]);
      case "gif_search": return Promise.resolve([bannedRemote]);
      case "favorite_gif": return new Promise<Gif>((res) => { resolveFavorite = res; });
      default: return Promise.resolve(undefined);
    }
  });

  render(<App />);

  fireEvent.change(getInput(), { target: { value: "gif" } });
  await waitFor(() => expect(screen.queryByText("Search GIFs")).not.toBeNull());
  fireEvent.keyDown(window, { key: "Enter" });
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("gif_library"));

  fireEvent.change(getInput(), { target: { value: "band" } });
  await waitFor(() => expect(screen.queryByText("banned gif")).not.toBeNull(), { timeout: 2000 });

  fireEvent.keyDown(window, { key: "ArrowDown" });
  fireEvent.keyDown(window, { key: "Enter", metaKey: true });
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("favorite_gif", { gif: bannedRemote }));

  // leave gif mode before the favorite promise resolves — the pending id is
  // still armed at this point
  fireEvent.keyDown(window, { key: "Escape" });
  await waitFor(() => expect(getInput().value).toBe(""));

  // move off index 0 so a leaked selectIndex:0 (the saved gif's post-resolve
  // position) wouldn't be masked by already sitting there
  fireEvent.keyDown(window, { key: "ArrowDown" });
  const selectedBefore = document.querySelector("li > div.bg-sel")?.textContent;

  // now let the stale favorite resolve; the gif list state update it triggers
  // is exactly what used to fire the leaked selectIndex dispatch. Draining a
  // few extra microtask turns inside act() is needed because the chain runs
  // through two chained promises (useGifs' internal .then, then the awaited
  // gifActions.favorite wrapper in App) before the ref is set and a render
  // with the new gifResults reference lets the effect see it.
  await act(async () => {
    resolveFavorite(bannedSavedLocal);
    for (let i = 0; i < 5; i++) await Promise.resolve();
  });

  const selectedAfter = document.querySelector("li > div.bg-sel")?.textContent;
  expect(selectedAfter).toBe(selectedBefore);
});
