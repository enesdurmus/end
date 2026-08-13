// Mirrors the `Clip` enum in src-tauri/src/clipboard.rs (serde tag = "kind").
// An image carries a path, not pixels: the history file is read whole at startup.
export type Clip =
  | { kind: "text"; text: string }
  | { kind: "image"; path: string; name: string; width: number; height: number; bytes: number }
  | { kind: "files"; paths: string[] };

export type Result = {
  id: string;
  type: "app" | "file" | "clipboard" | "snippet" | "command" | "language" | "translation" | "gif";
  title: string;
  aliases?: string[]; // extra terms the fuzzy match considers (for commands)
  subtitle?: string;
  body?: string; // full text for the translation preview pane
  clip?: Clip;   // clipboard rows: what the preview pane renders, per kind
  icon?: string;
  run: () => void | Promise<void>;
  altRun?: () => void | Promise<void>; // ⌘Enter; falls back to run()
};

export type TranslationEntry = {
  source: string;
  translated: string;
  from: string;
  to: string;
};

export type Gif = {
  id: string;
  title: string;
  preview: string; // thumbnail source: an https URL, or a local path
  url: string;     // what gets copied: full-size URL, or the local path
  source: "local" | "remote";
};
