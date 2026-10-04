import { LANGUAGES, languageName } from "../lib/languages";
import { Button } from "./ui/Button";
import { Icon } from "./ui/Icon";
import { Select } from "./ui/Select";

// "from  ⇄  to" for the translate screen
export function LangBar({ source, target, detected, onSource, onTarget, onSwap }: {
  source: string;
  target: string;
  detected?: string; // what auto-detect resolved to, once a translation has landed
  onSource: (code: string) => void;
  onTarget: (code: string) => void;
  onSwap: () => void;
}) {
  const languages = LANGUAGES.map((l) => ({ value: l.code, label: l.name }));
  const auto = { value: "auto", label: detected ? `Detect (${languageName(detected)})` : "Detect language" };
  return (
    <div className="flex-none flex items-center gap-2.5 px-4 pt-4 pb-3.5">
      <Select className="w-[210px]" value={source} options={[auto, ...languages]} onChange={onSource} />
      <Button className="h-[34px] w-[34px] p-0!" title="Swap languages (⌘S)" aria-label="Swap languages" onClick={onSwap}>
        <Icon name="swap" size={16} />
      </Button>
      <Select className="w-[210px]" value={target} options={languages} onChange={onTarget} />
    </div>
  );
}
