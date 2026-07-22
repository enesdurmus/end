import { ReactNode } from "react";

export function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="mb-3">
      <div className="text-[13px] mb-1">{label}</div>
      {children}
    </div>
  );
}
