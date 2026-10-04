import { ReactNode } from "react";

// label + control, with the hint/error line every settings row shares
export function Field({ label, hint, error, children }: {
  label: string; hint?: string; error?: string; children: ReactNode;
}) {
  return (
    <div className="mb-3">
      <div className="text-[13px] text-[#aab0dc] mb-1.5">{label}</div>
      {children}
      {error ? <div className="text-danger text-xs mt-1">{error}</div>
        : hint && <div className="text-fg-dim text-xs mt-1">{hint}</div>}
    </div>
  );
}
