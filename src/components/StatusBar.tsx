import { Kbd } from "./ui/Kbd";

export function StatusBar({ count }: { count: number }) {
  return (
    <div className="flex-none flex items-center justify-between px-3.5 py-2 border-t border-hair text-xs text-fg-dim">
      <span>{count} results</span>
      <span className="flex items-center gap-3">
        <span>Open<Kbd>↵</Kbd></span>
        <span>Close<Kbd>esc</Kbd></span>
      </span>
    </div>
  );
}
