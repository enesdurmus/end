import { ReactNode } from "react";
import { Icon, IconName } from "./Icon";

const GLYPH: Record<string, IconName> = {
  "↑": "up", "↓": "down", "←": "left", "→": "right", "↵": "enter", "⌫": "backspace", "⌘": "command",
};

// Key glyphs are drawn as icons: system fonts render ⌘ ↵ ⌫ unevenly (or not at all on Linux)
function Glyphs({ text }: { text: string }) {
  return text.split(/([↑↓←→↵⌫⌘])/).filter(Boolean).map((t, i) =>
    GLYPH[t] ? <Icon key={i} name={GLYPH[t]} /> : <span key={i} className="whitespace-pre">{t}</span>
  );
}

export function Kbd({ children }: { children: ReactNode }) {
  return (
    <span className="inline-flex items-center justify-center gap-1.5 min-w-[26px] h-6 px-2 surface-chip rounded-md text-xs">
      {typeof children === "string" ? <Glyphs text={children} /> : children}
    </span>
  );
}
