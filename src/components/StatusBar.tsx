import { Kbd } from "./ui/Kbd";

export function StatusBar({ left, hints }: { left: string; hints: [string, string][] }) {
  return (
    <div className="flex-none flex items-center gap-5 px-4 py-2.5 border-t border-hair text-[13px] text-[#aeb3d6]">
      {hints.map(([label, key]) => (
        <span key={label} className="flex items-center gap-2">
          <Kbd>{key}</Kbd>{label}
        </span>
      ))}
      <span className="ml-auto text-fg-dim">{left}</span>
    </div>
  );
}
