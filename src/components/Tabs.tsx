export const TABS = ["All", "Apps", "Files", "Clipboard", "Commands", "Snippets"] as const;
export type Tab = (typeof TABS)[number];

export function Tabs({ active, onPick }: { active: Tab; onPick: (t: Tab) => void }) {
  return (
    <div className="flex-none flex items-center gap-1 px-4 pt-3.5 pb-3">
      {TABS.map((t) => (
        <button
          key={t}
          onClick={() => onPick(t)}
          className={
            "px-4 py-1.5 rounded-xl text-sm border border-transparent cursor-default transition-colors " +
            (t === active ? "surface-active text-fg" : "text-[#a5aacb] hover:text-fg")
          }
        >
          {t}
        </button>
      ))}
    </div>
  );
}
