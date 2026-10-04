import { useEffect, useRef } from "react";
import { Result } from "../types";
import { Kbd } from "./ui/Kbd";

const LABELS: Record<Result["type"], string> = {
  app: "Applications",
  file: "Files",
  clipboard: "Clipboard History",
  snippet: "Snippets",
  command: "Commands",
  language: "Languages",
  translation: "Translation History",
  gif: "GIFs",
};

// ponytail: hue from title so icons look distinct without real app icons
const hue = (s: string) => [...s].reduce((a, c) => a + c.charCodeAt(0), 0) % 360;

export function ResultList({ results, selected, onSelect }: { results: Result[]; selected: number; onSelect: (i: number) => void }) {
  const selRef = useRef<HTMLDivElement>(null);
  // a mouse-driven selection is already under the pointer; scrolling it would make the list jump
  const byMouse = useRef(false);
  useEffect(() => {
    if (byMouse.current) { byMouse.current = false; return; }
    selRef.current?.scrollIntoView({ block: "nearest" });
  }, [selected]);
  const mouse = (i: number, r: Result) => ({
    onMouseMove: () => { if (i !== selected) { byMouse.current = true; onSelect(i); } },
    onClick: () => { onSelect(i); r.run(); },
  });

  let lastType: Result["type"] | null = null;
  return (
    <ul className="scroll-thin list-none m-0 px-4 py-2 overflow-y-auto flex-1 min-h-0">
      {results.map((r, i) => {
        const header = r.type !== lastType ? LABELS[r.type] : null;
        lastType = r.type;
        return (
          <li key={r.id}>
            {header && (
              <div className="text-fg-dim text-[11px] font-medium tracking-[0.4px] px-2.5 pt-2.5 pb-1">
                {header}
              </div>
            )}
            <div
              ref={i === selected ? selRef : null}
              {...mouse(i, r)}
              className={
                "flex items-center gap-3 px-3.5 py-2 rounded-xl text-[17px] cursor-default border border-transparent " +
                (i === selected ? "surface-row" : "")
              }
            >
              {r.icon ? (
                <img className="flex-none w-7 h-7 rounded-md object-contain" src={r.icon} alt="" />
              ) : (
                <span
                  className="flex-none w-7 h-7 rounded-md grid place-items-center text-xs font-semibold text-white"
                  style={{ background: `hsl(${hue(r.title)} 55% 45%)` }}
                >
                  {r.title.trim().charAt(0).toUpperCase() || "?"}
                </span>
              )}
              <span className="flex-[0_1_auto] overflow-hidden text-ellipsis whitespace-nowrap">{r.title}</span>
              {r.subtitle && (
                <span className="text-fg-dim text-sm flex-1 overflow-hidden text-ellipsis whitespace-nowrap">
                  {r.subtitle}
                </span>
              )}
              {i === selected ? (
                <Kbd>↵ Open</Kbd>
              ) : (
                <span className="text-fg-dim text-xs flex-none">{r.type}</span>
              )}
            </div>
          </li>
        );
      })}
    </ul>
  );
}
