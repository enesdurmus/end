import { useEffect, useRef } from "react";
import { Result } from "../types";

const LABELS: Record<Result["type"], string> = {
  app: "Applications",
  file: "Files",
  clipboard: "Clipboard History",
  snippet: "Snippets",
  command: "Commands",
  language: "Languages",
  translation: "Translation History",
};

// ponytail: hue from title so icons look distinct without real app icons
const hue = (s: string) => [...s].reduce((a, c) => a + c.charCodeAt(0), 0) % 360;

export function ResultList({ results, selected }: { results: Result[]; selected: number }) {
  const selRef = useRef<HTMLDivElement>(null);
  useEffect(() => { selRef.current?.scrollIntoView({ block: "nearest" }); }, [selected]);

  let lastType: Result["type"] | null = null;
  return (
    <ul className="scroll-thin list-none m-0 p-2 overflow-y-auto flex-1 min-h-0">
      {results.map((r, i) => {
        const header = r.type !== lastType ? LABELS[r.type] : null;
        lastType = r.type;
        return (
          <li key={r.id}>
            {header && (
              <div className="text-fg-dim text-[11px] font-semibold tracking-[0.4px] px-2.5 pt-2.5 pb-1">
                {header}
              </div>
            )}
            <div
              ref={i === selected ? selRef : null}
              className={
                "flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-sm cursor-default " +
                (i === selected ? "bg-sel" : "")
              }
            >
              {r.icon ? (
                <img className="flex-none w-6 h-6 rounded-md object-contain" src={r.icon} alt="" />
              ) : (
                <span
                  className="flex-none w-6 h-6 rounded-md grid place-items-center text-xs font-semibold text-white"
                  style={{ background: `hsl(${hue(r.title)} 55% 45%)` }}
                >
                  {r.title.trim().charAt(0).toUpperCase() || "?"}
                </span>
              )}
              <span className="flex-[0_1_auto] overflow-hidden text-ellipsis whitespace-nowrap">{r.title}</span>
              {r.subtitle && (
                <span className="text-fg-dim text-xs flex-1 overflow-hidden text-ellipsis whitespace-nowrap">
                  {r.subtitle}
                </span>
              )}
              <span className="text-fg-dim text-[11px] flex-none">{r.type}</span>
            </div>
          </li>
        );
      })}
    </ul>
  );
}
