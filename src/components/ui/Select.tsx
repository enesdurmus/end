import { useEffect, useRef, useState } from "react";
import { Icon } from "./Icon";

export type Option = { value: string; label: string };

// A native <select> opens the OS menu, which ignores our styling, so the list is our own.
export function Select({ value, options, onChange, className = "" }: {
  value: string;
  options: Option[];
  onChange: (value: string) => void;
  className?: string;
}) {
  const [open, setOpen] = useState(false);
  const [hi, setHi] = useState(0); // highlighted row
  const root = useRef<HTMLDivElement>(null);
  const current = options.findIndex((o) => o.value === value);

  const pick = (v: string) => { onChange(v); setOpen(false); };

  useEffect(() => {
    if (!open) return;
    const away = (e: MouseEvent) => { if (!root.current?.contains(e.target as Node)) setOpen(false); };
    // capture + stopPropagation: while the list is open, Esc/Enter/arrows are its own,
    // not the screen's (Esc would otherwise leave Preferences)
    const key = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
      else if (e.key === "ArrowDown") setHi((h) => Math.min(h + 1, options.length - 1));
      else if (e.key === "ArrowUp") setHi((h) => Math.max(h - 1, 0));
      else if (e.key === "Enter") pick(options[hi].value);
      else return;
      e.preventDefault();
      e.stopPropagation();
    };
    document.addEventListener("mousedown", away);
    window.addEventListener("keydown", key, true);
    return () => {
      document.removeEventListener("mousedown", away);
      window.removeEventListener("keydown", key, true);
    };
  });

  return (
    <div ref={root} className={`relative ${className}`}>
      <button
        type="button"
        className="control w-full flex items-center justify-between gap-2 text-left cursor-pointer"
        onClick={() => { setHi(Math.max(current, 0)); setOpen(!open); }}
      >
        <span className="truncate">{options[current]?.label}</span>
        <Icon name="chevron" size={14} className="flex-none text-fg-dim" />
      </button>
      {open && (
        <ul className="popover scroll-thin absolute z-10 left-0 right-0 mt-1.5 max-h-56 overflow-y-auto p-1 m-0 list-none">
          {options.map((o, i) => (
            <li
              key={o.value}
              ref={i === hi ? (el) => el?.scrollIntoView({ block: "nearest" }) : undefined}
              onMouseMove={() => setHi(i)}
              onClick={() => pick(o.value)}
              className={
                "px-3 py-1.5 rounded-md text-[13px] cursor-default " +
                (i === hi ? "surface-row " : "") +
                (o.value === value ? "text-white" : "")
              }
            >
              {o.label}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
