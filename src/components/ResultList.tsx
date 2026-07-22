import { useEffect, useRef } from "react";
import { Result } from "../types";

const LABELS: Record<Result["type"], string> = {
  app: "Applications",
  file: "Files",
  clipboard: "Clipboard History",
  snippet: "Snippets",
  command: "Commands",
};

// ponytail: hue from title so icons look distinct without real app icons
const hue = (s: string) => [...s].reduce((a, c) => a + c.charCodeAt(0), 0) % 360;

export function ResultList({ results, selected }: { results: Result[]; selected: number }) {
  const selRef = useRef<HTMLLIElement>(null);
  useEffect(() => { selRef.current?.scrollIntoView({ block: "nearest" }); }, [selected]);

  let lastType: Result["type"] | null = null;
  return (
    <ul className="results">
      {results.map((r, i) => {
        const header = r.type !== lastType ? LABELS[r.type] : null;
        lastType = r.type;
        return (
          <li key={r.id}>
            {header && <div className="section">{header}</div>}
            <div
              ref={i === selected ? (selRef as any) : null}
              className={i === selected ? "row selected" : "row"}
            >
              {r.icon ? (
                <img className="icon" src={r.icon} alt="" />
              ) : (
                <span className="icon" style={{ background: `hsl(${hue(r.title)} 55% 45%)` }}>
                  {r.title.trim().charAt(0).toUpperCase() || "?"}
                </span>
              )}
              <span className="title">{r.title}</span>
              {r.subtitle && <span className="subtitle">{r.subtitle}</span>}
              <span className="badge">{r.type}</span>
            </div>
          </li>
        );
      })}
    </ul>
  );
}
