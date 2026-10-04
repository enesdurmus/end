import { ReactNode } from "react";

const P = {
  "↑": "M12 19V5M5 12l7-7 7 7",
  "↓": "M12 5v14M19 12l-7 7-7-7",
  "←": "M19 12H5M12 19l-7-7 7-7",
  "→": "M5 12h14M12 5l7 7-7 7",
  "↵": "M20 4v7a4 4 0 0 1-4 4H4M9 10l-5 5 5 5",
  "⌫": "M10 5a2 2 0 0 0-1.3.5l-6.4 5.7a1 1 0 0 0 0 1.5l6.4 5.7A2 2 0 0 0 10 19h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2zM12 9l6 6M18 9l-6 6",
  "⌘": "M15 6v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3",
} as const;

// Key glyphs are drawn as icons: system fonts render ⌘ ↵ ⌫ unevenly (or not at all on Linux)
function Glyphs({ text }: { text: string }) {
  return [...text].map((c, i) =>
    c in P ? (
      <svg key={i} aria-hidden="true" width="14" height="14" viewBox="0 0 24 24" fill="none"
        stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d={P[c as keyof typeof P]} />
      </svg>
    ) : (
      <span key={i} className="whitespace-pre">{c}</span>
    )
  );
}

export function Kbd({ children }: { children: ReactNode }) {
  return (
    <span className="inline-flex items-center justify-center gap-1.5 min-w-[28px] h-7 px-2 surface-chip rounded-md text-xs">
      {typeof children === "string" ? <Glyphs text={children} /> : children}
    </span>
  );
}
