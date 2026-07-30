import { beforeEach, expect, test, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useApps } from "./useApps";
import { runActions } from "../lib/actions";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

beforeEach(() => invoke.mockReset());

test("loads apps, then patches in real icons as they resolve", async () => {
  invoke.mockImplementation((cmd: string, args?: { path: string }) => {
    if (cmd === "list_apps") return Promise.resolve([{ name: "Safari", path: "/A/Safari.app" }]);
    if (cmd === "app_icon") return Promise.resolve(args?.path === "/A/Safari.app" ? "data:icon" : null);
    return Promise.resolve(null);
  });

  const { result } = renderHook(() => useApps(runActions));

  // first: app appears without an icon
  await waitFor(() => expect(result.current).toHaveLength(1));
  expect(result.current[0]).toMatchObject({ id: "app:/A/Safari.app", title: "Safari" });

  // then: icon is patched in
  await waitFor(() => expect(result.current[0].icon).toBe("data:icon"));
});

test("apps with no icon stay iconless", async () => {
  invoke.mockImplementation((cmd: string) =>
    Promise.resolve(cmd === "list_apps" ? [{ name: "X", path: "/X.app" }] : null)
  );
  const { result } = renderHook(() => useApps(runActions));
  await waitFor(() => expect(result.current).toHaveLength(1));
  expect(result.current[0].icon).toBeUndefined();
});
