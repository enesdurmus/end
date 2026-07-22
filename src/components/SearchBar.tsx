import { Ref } from "react";

export function SearchBar({
  value,
  onChange,
  placeholder = "Search...",
  badge,
  inputRef,
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  badge?: string;
  inputRef?: Ref<HTMLInputElement>;
}) {
  return (
    <div className="flex-none flex items-center gap-3 px-5 border-b border-hair">
      <svg aria-hidden="true" className="text-fg-dim flex-none" width="18" height="18" viewBox="0 0 24 24" fill="none"
        stroke="currentColor" strokeWidth="2" strokeLinecap="round">
        <circle cx="11" cy="11" r="7" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      {badge && (
        <span className="flex-none px-2.5 py-[3px] rounded-md text-[13px] font-medium bg-sel text-fg whitespace-nowrap">
          {badge}
        </span>
      )}
      <input
        ref={inputRef}
        className="w-full border-0 outline-none bg-transparent text-fg text-xl py-4 font-normal placeholder:text-fg-dim"
        autoFocus
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}
