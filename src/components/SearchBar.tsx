export function SearchBar({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  return (
    <input
      className="searchbar"
      autoFocus
      placeholder="Ara..."
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  );
}
