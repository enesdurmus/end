import { check } from "@tauri-apps/plugin-updater";
import { ask } from "@tauri-apps/plugin-dialog";
import { relaunch } from "@tauri-apps/plugin-process";

export async function checkForUpdates(): Promise<void> {
  const update = await check();
  if (!update) return;

  const shouldInstall = await ask(
    `End ${update.version} is available. Restart now to install it?`,
    { title: "Update available", kind: "info" }
  );
  if (!shouldInstall) return;

  await update.downloadAndInstall();
  await relaunch();
}
