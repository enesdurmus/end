import { Icon, IconName } from "./ui/Icon";

export const TABS = ["All", "Apps", "Files", "Clipboard", "Commands", "Snippets"] as const;
export type Tab = (typeof TABS)[number];

const ICON: Record<Tab, IconName> = {
  All: "grid", Apps: "grid", Files: "file", Clipboard: "clipboard", Commands: "terminal", Snippets: "code",
};

export function Tabs({ active, onPick, onSettings }: { active: Tab; onPick: (t: Tab) => void; onSettings: () => void }) {
  return (
    <div className="flex-none flex items-center gap-2.5 px-4 pt-4 pb-3.5">
      {TABS.map((t) => (
        <button
          key={t}
          onClick={() => onPick(t)}
          className={
            "inline-flex items-center gap-2 h-[31px] px-3.5 rounded-[10px] text-[12.5px] cursor-default transition-colors " +
            (t === active
              ? "surface-active text-white"
              : "surface-chip hover:text-fg hover:brightness-125")
          }
        >
          <Icon name={ICON[t]} size={15} />
          {t}
        </button>
      ))}
      <button
        onClick={onSettings}
        title="Preferences"
        aria-label="Preferences"
        className="ml-auto grid place-items-center h-[31px] w-[31px] rounded-[10px] surface-chip cursor-default hover:text-fg hover:brightness-125"
      >
        <Icon name="settings" size={16} />
      </button>
    </div>
  );
}
