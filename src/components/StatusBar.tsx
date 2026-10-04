import { Kbd } from "./ui/Kbd";

export function StatusBar({ left, hints }: { left: string; hints: [string, string][] }) {
  return (
    <div className="flex-none flex items-center gap-6 px-4 h-[42px] text-xs text-[#c0c6dc]">
      {hints.map(([label, key]) => (
        <span key={label} className="flex items-center gap-2.5">
          <Kbd>{key}</Kbd>{label}
        </span>
      ))}
      <span className="ml-auto text-fg-dim">{left}</span>
    </div>
  );
}
