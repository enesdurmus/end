import { useEffect, useRef } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Result } from "../types";

// Local rows carry a filesystem path; the webview can only load it through
// Tauri's asset protocol. Remote rows are already https and pass through.
const src = (s: string) => (s.startsWith("http") ? s : convertFileSrc(s));

export function GifList({ results, selected }: { results: Result[]; selected: number }) {
  const selRef = useRef<HTMLDivElement>(null);
  useEffect(() => { selRef.current?.scrollIntoView({ block: "nearest" }); }, [selected]);

  return (
    <ul className="scroll-thin list-none m-0 p-2 overflow-y-auto flex-1 min-h-0">
      {results.map((r, i) => (
        <li key={r.id}>
          <div
            ref={i === selected ? selRef : null}
            className={
              "flex items-center gap-3 px-2.5 py-2 rounded-lg text-sm cursor-default " +
              (i === selected ? "bg-sel" : "")
            }
          >
            <img
              className="flex-none w-16 h-12 rounded-md object-cover bg-sel"
              src={r.icon ? src(r.icon) : undefined}
              alt=""
              loading="lazy"
            />
            <span className="flex-1 overflow-hidden text-ellipsis whitespace-nowrap">
              {r.title}
            </span>
            <span className="text-fg-dim text-[11px] flex-none">{r.subtitle}</span>
          </div>
        </li>
      ))}
    </ul>
  );
}
