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
    <div className="searchbox">
      <svg className="glyph" width="18" height="18" viewBox="0 0 24 24" fill="none"
        stroke="currentColor" strokeWidth="2" strokeLinecap="round">
        <circle cx="11" cy="11" r="7" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      {badge && <span className="pill">{badge}</span>}
      <input
        ref={inputRef}
        className="searchbar"
        autoFocus
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}
