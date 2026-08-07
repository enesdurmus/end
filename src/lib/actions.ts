import { invoke } from "@tauri-apps/api/core";
import { Gif, TranslationEntry } from "../types";

// Injected into result mappers so those stay pure/testable.
export type RunActions = {
  paste: (text: string) => Promise<void> | void;
  open: (path: string) => Promise<void> | void;
  copy: (text: string) => Promise<void> | void;
  record: (entry: TranslationEntry) => Promise<void> | void;
  pasteGif: (gif: Gif) => Promise<void> | void;
  favorite: (gif: Gif) => Promise<Gif> | void;
  openGifDir: () => Promise<void> | void;
};

// Hiding the window and handing focus back belongs to src-tauri/src/focus.rs; doing it
// here raced the paste keystroke. Copy goes through arboard for the same reason —
// navigator.clipboard needs a focused document we're in the middle of giving away.
export const runActions: RunActions = {
  paste: (text) => invoke("paste_text", { text }),
  open: (path) => invoke("open_path", { path }),
  copy: (text) => invoke("write_clipboard", { text }),
  record: (entry) => invoke("record_translation", { entry }),
  pasteGif: (gif) => invoke("paste_gif", { gif }),
  favorite: (gif) => invoke("favorite_gif", { gif }),
  openGifDir: () => invoke("open_gif_dir"),
};
