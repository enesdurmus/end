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
    <div className="flex-none m-4 mb-0 flex items-center gap-3 px-4 rounded-2xl surface-field transition-colors focus-within:border-[#7c4dff]/70">
      <svg aria-hidden="true" className="text-[#c7cbee] flex-none" width="22" height="22" viewBox="0 0 24 24" fill="none"
        stroke="currentColor" strokeWidth="1.6" strokeLinecap="round">
        <circle cx="11" cy="11" r="7" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      {badge && (
        <span className="flex-none px-2 py-[2px] rounded-md text-[12px] font-medium bg-sel text-fg whitespace-nowrap">
          {badge}
        </span>
      )}
      <input
        ref={inputRef}
        className="w-full border-0 outline-none bg-transparent text-fg text-lg py-3.5 font-normal placeholder:text-[#6e74a8]"
        autoFocus
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}
