import { Result } from "../types";
import { ResultList } from "./ResultList";
import { ClipPreview } from "./ClipPreview";
import { Split } from "./ui/Split";

export function ClipboardView({ results, selected, onSelect }: { results: Result[]; selected: number; onSelect: (i: number) => void }) {
  const clip = results[selected]?.clip;
  const body = results[selected]?.body;
  return (
    <Split
      left={<ResultList results={results} selected={selected} onSelect={onSelect} />}
      right={
        clip ? (
          <ClipPreview clip={clip} />
        ) : body ? (
          <pre className="m-0 font-mono text-[13px] leading-[1.5] text-fg whitespace-pre-wrap break-words">
            {body.slice(0, 5000)}
          </pre>
        ) : (
          <div className="text-fg-dim text-[13px] grid place-items-center h-full">No selection</div>
        )
      }
    />
  );
}
