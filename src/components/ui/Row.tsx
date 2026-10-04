import { ReactNode, Ref } from "react";

// One two-line list row (icon, title, subtitle, trailing slot) shared by every list.
export function Row({ rowRef, selected, icon, title, sub, right, onMouseMove, onClick }: {
  rowRef?: Ref<HTMLDivElement>;
  selected: boolean;
  icon: ReactNode;
  title: string;
  sub?: string;
  right?: ReactNode;
  onMouseMove: () => void;
  onClick: () => void;
}) {
  return (
    <div
      ref={rowRef}
      onMouseMove={onMouseMove}
      onClick={onClick}
      className={
        "flex items-center gap-3 px-3 py-[7px] rounded-lg cursor-default border border-transparent " +
        (selected ? "surface-row" : "")
      }
    >
      {icon}
      <div className="flex-1 min-w-0">
        <div className="text-[14.5px] leading-tight truncate">{title}</div>
        {sub && <div className="text-[12.5px] leading-tight mt-1 text-fg-dim truncate">{sub}</div>}
      </div>
      {right}
    </div>
  );
}
