import { Dispatch } from "react";
import { Result } from "./types";
import { NavAction } from "./lib/navigation";

// Mode-switch commands, Raycast-style: fuzzy-searchable in root, Enter switches view.
export function buildCommands(dispatch: Dispatch<NavAction>, loadClips: () => void): Result[] {
  return [
    {
      id: "cmd:clipboard", type: "command", title: "Clipboard History",
      subtitle: "Browse and paste clipboard history", aliases: ["clipboard", "clips"],
      run: () => { dispatch({ type: "goMode", mode: "clipboard" }); loadClips(); },
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
