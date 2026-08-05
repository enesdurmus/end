import { Dispatch } from "react";
import { Result } from "./types";
import { NavAction } from "./lib/navigation";

export function buildCommands(
  dispatch: Dispatch<NavAction>,
  loadClips: () => void,
  loadHistory: () => void
): Result[] {
  return [
    {
      id: "cmd:clipboard", type: "command", title: "Clipboard History",
      subtitle: "Browse and paste clipboard history", aliases: ["clipboard", "clips"],
      run: () => { dispatch({ type: "goMode", mode: "clipboard" }); loadClips(); },
    },
    {
      id: "cmd:translate", type: "command", title: "Translate",
      subtitle: "Translate text without leaving the launcher",
      aliases: ["translate", "tr", "çevir", "cevir"],
      run: () => { dispatch({ type: "goMode", mode: "translate" }); loadHistory(); },
    },
    {
      id: "cmd:files", type: "command", title: "Search Files",
      subtitle: "Find files by name", aliases: ["files", "find"],
      run: () => dispatch({ type: "goMode", mode: "files" }),
    },
    {
      id: "cmd:snippets", type: "command", title: "Manage Snippets",
      subtitle: "Create and edit snippets", aliases: ["snippets"],
      run: () => dispatch({ type: "manage" }),
    },
  ];
}
