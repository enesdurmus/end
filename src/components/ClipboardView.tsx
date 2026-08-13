import { Result } from "../types";
import { ResultList } from "./ResultList";
import { ClipPreview } from "./ClipPreview";

export function ClipboardView({ results, selected }: { results: Result[]; selected: number }) {
  const clip = results[selected]?.clip;
  const body = results[selected]?.body;
  return (
    <div className="flex flex-1 min-h-0">
      <div className="flex-none w-[42%] border-r border-hair flex flex-col min-h-0">
        <ResultList results={results} selected={selected} />
      </div>
      <div className="scroll-thin flex-1 min-w-0 px-[18px] py-4 overflow-y-auto">
        {clip ? (
          <ClipPreview clip={clip} />
        ) : body ? (
          <pre className="m-0 font-mono text-[13px] leading-[1.5] text-fg whitespace-pre-wrap break-words">
            {body.slice(0, 5000)}
          </pre>
        ) : (
          <div className="text-fg-dim text-[13px] grid place-items-center h-full">No selection</div>
        )}
      </div>
    </div>
  );
}
