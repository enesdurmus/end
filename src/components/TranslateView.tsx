import { TranslationEntry } from "../types";

// The blank-query state uses ClipboardView (history list + preview) instead.
export function TranslateView({
  entry,
  loading,
  error,
}: {
  entry: TranslationEntry | null;
  loading: boolean;
  error: string;
}) {
  return (
    <div className="card scroll-thin flex-1 min-h-0 mx-4 mb-3 p-5 overflow-y-auto">
      {error ? (
        <div className="text-danger text-[13px]">{error}</div>
      ) : entry ? (
        <p className="m-0 text-[15px] leading-[1.5] text-fg whitespace-pre-wrap break-words">
          {entry.translated}
        </p>
      ) : (
        <div className="text-fg-dim text-[13px]">{loading ? "Translating…" : ""}</div>
      )}
    </div>
  );
}
