export type Result = {
  id: string;
  type: "app" | "file" | "clipboard" | "snippet" | "command" | "language" | "translation";
  title: string;
  aliases?: string[]; // extra terms the fuzzy match considers (for commands)
  subtitle?: string;
  body?: string; // full text for the clipboard preview pane
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
