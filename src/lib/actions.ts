import { invoke } from "@tauri-apps/api/core";
import { Clip, Gif, TranslationEntry } from "../types";

// Injected into result mappers so those stay pure/testable.
export type RunActions = {
  paste: (text: string) => Promise<void> | void;
  // A history entry of any kind; images and files go back on the clipboard as
  // file references, so pasting them attaches the file rather than its path.
  pasteClip: (clip: Clip) => Promise<void> | void;
  open: (path: string) => Promise<void> | void;
  copy: (text: string) => Promise<void> | void;
  record: (entry: TranslationEntry) => Promise<void> | void;
  pasteGif: (gif: Gif) => Promise<void> | void;
  // Resolves to the new local Gif favorite_gif downloaded — useGifs needs it to
  // replace the remote row it came from instead of re-listing the whole folder.
  favorite: (gif: Gif) => Promise<Gif>;
};

// Hiding and restoring focus belongs to src-tauri/src/focus.rs — doing it here
// raced the paste keystroke. Copy goes through arboard for the same reason.
export const runActions: RunActions = {
  paste: (text) => invoke("paste_text", { text }),
  pasteClip: (clip) => invoke("paste_clip", { clip }),
  open: (path) => invoke("open_path", { path }),
  copy: (text) => invoke("write_clipboard", { text }),
  record: (entry) => invoke("record_translation", { entry }),
  pasteGif: (gif) => invoke("paste_gif", { gif }),
  favorite: (gif) => invoke<Gif>("favorite_gif", { gif }),
};
