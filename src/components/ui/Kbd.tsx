import { ReactNode } from "react";

export function Kbd({ children }: { children: ReactNode }) {
  return (
    <span className="inline-block min-w-[18px] text-center px-[5px] py-px ml-1.5 bg-white/10 rounded text-[11px] text-fg">
      {children}
    </span>
  );
}
