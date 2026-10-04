import { convertFileSrc } from "@tauri-apps/api/core";
import { Clip } from "../types";
import { basename } from "../lib/results";

// One clipboard entry, rendered per kind. Only the selected row reaches this
// component, so exactly one image is ever decoded — which is why list rows
// deliberately carry no thumbnails.
export function ClipPreview({ clip }: { clip: Clip }) {
  if (clip.kind === "text") {
    return (
      <pre className="m-0 font-mono text-[13px] leading-[1.5] text-fg whitespace-pre-wrap break-words">
        {clip.text.slice(0, 5000)}
      </pre>
    );
  }

  if (clip.kind === "image") {
    return (
      <div className="h-full flex flex-col gap-2 min-h-0">
        <img
          src={convertFileSrc(clip.path)}
          alt=""
          className="flex-1 min-h-0 w-full object-contain rounded-lg bg-[#7d78ff]/10"
        />
        <div className="flex-none text-fg-dim text-xs text-center">
          {clip.width} × {clip.height}
        </div>
      </div>
    );
  }

  return (
    <ul className="list-none m-0 p-0 flex flex-col gap-2">
      {clip.paths.map((p) => (
        <li key={p} className="min-w-0">
          <div className="text-[13px] text-fg overflow-hidden text-ellipsis whitespace-nowrap">
            {basename(p)}
          </div>
          <div className="text-fg-dim text-xs break-all">{p}</div>
        </li>
      ))}
    </ul>
  );
}
