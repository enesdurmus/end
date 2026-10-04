import { Ref } from "react";
import { Icon } from "./ui/Icon";
import { Kbd } from "./ui/Kbd";

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
    <div className="flex-none mx-4 mt-3.5 h-[42px] flex items-center gap-3 px-3.5 rounded-[10px] surface-field transition-colors focus-within:brightness-125">
      <Icon name="search" size={20} className="text-[#c0c6dc] flex-none" />
      {badge && (
        <span className="flex-none px-2 py-[2px] rounded-md text-[12px] font-medium bg-sel text-fg whitespace-nowrap">
          {badge}
        </span>
      )}
      <input
        ref={inputRef}
        className="w-full h-full border-0 outline-none bg-transparent text-fg text-[15px] font-normal placeholder:text-[#7e87a8]"
        autoFocus
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
      <Kbd>⌘K</Kbd>
    </div>
  );
}
