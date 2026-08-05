import { Kbd } from "./ui/Kbd";

export function StatusBar({ left, hints }: { left: string; hints: [string, string][] }) {
  return (
    <div className="flex-none flex items-center justify-between px-3.5 py-2 border-t border-hair text-xs text-fg-dim">
      <span>{left}</span>
      <span className="flex items-center gap-3">
        {hints.map(([label, key]) => (
          <span key={label}>{label}<Kbd>{key}</Kbd></span>
        ))}
      </span>
    </div>
  );
}
