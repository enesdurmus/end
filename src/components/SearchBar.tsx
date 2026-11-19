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
    <div className="flex-none mx-3 mt-3 h-[36px] flex items-center gap-2.5 px-3 rounded-md surface-field transition-colors focus-within:brightness-125">
      <span className="flex-none text-accent font-semibold">&gt;</span>
      {badge && (
        <span className="flex-none px-2 py-[2px] rounded text-[11px] font-medium bg-sel text-fg whitespace-nowrap">
          {badge}
        </span>
      )}
      <input
        ref={inputRef}
        className="w-full h-full border-0 outline-none bg-transparent text-fg text-[13.5px] font-normal placeholder:text-[#5f6678]"
        autoFocus
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}
