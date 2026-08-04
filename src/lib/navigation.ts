// Pure state machine for the launcher UI. No DOM, no Tauri — fully testable.
// Side effects (e.g. loading clipboard history) live in the caller, not here.

export type Mode = "root" | "clipboard" | "files";

export type NavState = {
  mode: Mode;
  query: string;
  selected: number;
  managing: boolean;
};

export type NavAction =
  | { type: "setQuery"; query: string }
  | { type: "move"; delta: number; max: number }
  | { type: "goMode"; mode: Exclude<Mode, "root"> }
  | { type: "goRoot" }
  | { type: "manage" }
  | { type: "closeManage" };

export const initialNav: NavState = { mode: "root", query: "", selected: 0, managing: false };

export function navReducer(s: NavState, a: NavAction): NavState {
  switch (a.type) {
    // typing resets the highlight to the top result
    case "setQuery":
      return { ...s, query: a.query, selected: 0 };
    // clamp selection into [0, max-1]; empty list stays at 0
    case "move":
      return { ...s, selected: Math.max(0, Math.min(s.selected + a.delta, a.max - 1)) };
    case "goMode":
      return { ...s, mode: a.mode, query: "", selected: 0, managing: false };
    case "goRoot":
      return { ...s, mode: "root", query: "", selected: 0, managing: false };
    case "manage":
      return { ...s, managing: true, query: "" };
    case "closeManage":
      return { ...s, managing: false };
  }
}
