import { ReactNode } from "react";

// list on the left, detail card on the right
export function Split({ left, right }: { left: ReactNode; right: ReactNode }) {
  return (
    <div className="flex flex-1 min-h-0 gap-5 px-4 pb-3">
      <div className="flex-none w-[48%] min-h-0 flex flex-col">{left}</div>
      <div className="card scroll-thin flex-1 min-w-0 min-h-0 overflow-y-auto p-5">{right}</div>
    </div>
  );
}
