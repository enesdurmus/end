import { Meaning, Result, TranslationEntry } from "../types";
import { ResultList } from "./ResultList";
import { Split } from "./ui/Split";

// Every rendering of what was typed on the left (the translation, then its alternatives);
// the selected one large on the right, with dictionary meanings below it.
export function TranslateView({ results, selected, onSelect, entry, meanings, loading, error }: {
  results: Result[];
  selected: number;
  onSelect: (i: number) => void;
  entry: TranslationEntry | null;
  meanings: Meaning[];
  loading: boolean;
  error: string;
}) {
  const body = results[selected]?.body;
  return (
    <Split
      left={<ResultList results={results} selected={selected} onSelect={onSelect} />}
      right={
        error ? (
          <div className="text-danger text-[13px]">{error}</div>
        ) : entry && body ? (
          <>
            <div className="text-fg-dim text-[13px] mb-3 whitespace-pre-wrap break-words">{entry.source}</div>
            <p className="m-0 text-[17px] leading-[1.5] text-fg whitespace-pre-wrap break-words">{body}</p>
            {meanings.length > 0 && (
              <div className="mt-6">
                <div className="text-sm text-[#c0c6dc] mb-2">Meanings</div>
                {meanings.map((m) => (
                  <div key={m.pos} className="grid grid-cols-[88px_1fr] gap-3 py-[7px] text-[12.5px]">
                    <span className="text-[#b4bbd3]">{m.pos}</span>
                    <span className="text-fg-dim">{m.terms.join(", ")}</span>
                  </div>
                ))}
              </div>
            )}
          </>
        ) : (
          <div className="text-fg-dim text-[13px]">{loading ? "Translating…" : ""}</div>
        )
      }
    />
  );
}
