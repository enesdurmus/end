import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { TranslationEntry } from "../types";

// The single Tauri side-effect boundary for running a Result.
// Injected into result mappers so those stay pure/testable.
export type RunActions = {
  paste: (text: string) => Promise<void> | void;
  open: (path: string) => Promise<void> | void;
  copy: (text: string) => Promise<void> | void;
  record: (entry: TranslationEntry) => Promise<void> | void;
};

export const runActions: RunActions = {
  paste: async (text) => { getCurrentWindow().hide(); await invoke("paste_text", { text }); },
  open: async (path) => { await invoke("open_path", { path }); getCurrentWindow().hide(); },
  // ponytail: arboard, not navigator.clipboard — the web API needs a focused
  // document, which a launcher that hides itself cannot promise. Hide first,
  // like paste, so a failed write can't strand the window open.
  copy: async (text) => { getCurrentWindow().hide(); await invoke("write_clipboard", { text }); },
  record: async (entry) => { await invoke("record_translation", { entry }); },
};
