// Pure state machine for the launcher UI. No DOM, no Tauri — fully testable.
// Side effects (e.g. loading clipboard history) live in the caller, not here.

// ⌘1–⌘9 run the first nine rows of any list that shows chips
export const ROW_SHORTCUTS = 9;

export type Mode = "root" | "clipboard" | "files" | "translate" | "gif";

// What fills the panel. Everything but "launcher" is a full-panel form with no result list.
export type Screen = "launcher" | "snippets" | "settings";

export type NavState = {
  mode: Mode;
  query: string;
  selected: number;
  screen: Screen;
  picking: boolean;    // language picker is showing in place of the results
  source: string;      // "auto" until the user pins it via swap
  target: string;
  savedQuery: string;  // text held while the picker borrows the search box
};

export type NavAction =
  | { type: "setQuery"; query: string }
  | { type: "move"; delta: number; max: number }
  | { type: "goMode"; mode: Exclude<Mode, "root"> }
  | { type: "goRoot" }
  | { type: "manage" }
  | { type: "openSettings" }
  | { type: "setLangs"; source: string; target: string }
  | { type: "pickLang" }
  | { type: "setTarget"; code: string }
  | { type: "cancelPick" }
  | { type: "swap"; detected: string }
  | { type: "selectIndex"; index: number };

export const initialNav: NavState = {
  mode: "root",
  query: "",
  selected: 0,
  screen: "launcher",
  picking: false,
  source: "auto",
  target: "en",
  savedQuery: "",
};

export function navReducer(s: NavState, a: NavAction): NavState {
  switch (a.type) {
    // typing resets the highlight to the top result
    case "setQuery":
      return { ...s, query: a.query, selected: 0 };
    // clamp selection into [0, max-1]; empty list stays at 0
    case "move":
      return { ...s, selected: Math.max(0, Math.min(s.selected + a.delta, a.max - 1)) };
    case "goMode":
      return { ...s, mode: a.mode, query: "", selected: 0, screen: "launcher", picking: false, savedQuery: "" };
    case "goRoot":
      return { ...s, mode: "root", query: "", selected: 0, screen: "launcher", picking: false, savedQuery: "" };
    case "manage":
      return { ...s, screen: "snippets", query: "" };
    case "openSettings":
      return { ...s, screen: "settings", picking: false, query: "", selected: 0 };
    case "setLangs":
      return { ...s, source: a.source, target: a.target };
    // the picker borrows the search box, so park the typed text until it closes
    case "pickLang":
      return { ...s, picking: true, savedQuery: s.query, query: "", selected: 0 };
    case "setTarget":
      return { ...s, picking: false, target: a.code, query: s.savedQuery, savedQuery: "", selected: 0 };
    case "cancelPick":
      return { ...s, picking: false, query: s.savedQuery, savedQuery: "", selected: 0 };
    // "auto" has no direction to flip, so the detected language stands in for it
    case "swap": {
      const from = s.source === "auto" ? a.detected : s.source;
      return { ...s, source: s.target, target: from, selected: 0 };
    }
    // used after favouriting a gif: the caller resolves *where* the new local
    // row landed (it may be re-sorted by the active query) before dispatching,
    // so this only ever applies an already-resolved position
    case "selectIndex":
      return { ...s, selected: a.index };
  }
}
