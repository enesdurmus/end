import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Result } from "../types";
import { RunActions } from "../lib/actions";
import { appToResult, RawApp } from "../lib/results";

export function useApps(actions: RunActions): Result[] {
  const [apps, setApps] = useState<Result[]>([]);

  useEffect(() => {
    invoke<RawApp[]>("list_apps").then((list) => {
      setApps(list.map((a) => appToResult(a, actions)));
      list.forEach((a) =>
        invoke<string | null>("app_icon", { path: a.path }).then((icon) => {
          if (!icon) return;
          setApps((prev) => prev.map((r) => (r.id === "app:" + a.path ? { ...r, icon } : r)));
        })
      );
    });
    // actions is a stable module const; intentionally run once
  }, []);

  return apps;
}
