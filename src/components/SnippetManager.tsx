import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "./ui/Window";
import { Button } from "./ui/Button";
import { Input } from "./ui/Input";

type Snippet = { keyword: string; text: string };

export function SnippetManager({ onClose }: { onClose: () => void }) {
  const [items, setItems] = useState<Snippet[]>([]);
  useEffect(() => { invoke<Snippet[]>("list_snippets").then(setItems); }, []);
  const persist = (next: Snippet[]) => { setItems(next); invoke("save_snippets", { items: next }); };
  return (
    <Window variant="floating">
      <div className="scroll-thin h-full overflow-y-auto p-5">
      <h3 className="m-0 mb-4 text-[15px] font-medium">Snippets</h3>
      {items.map((s, i) => (
        <div key={i} className="flex gap-2 mb-2">
          <Input value={s.keyword} placeholder="keyword"
            onChange={(e) => { const n = [...items]; n[i] = { ...s, keyword: e.target.value }; setItems(n); }}
            onBlur={() => persist(items)} />
          <Input value={s.text} placeholder="text" className="flex-1"
            onChange={(e) => { const n = [...items]; n[i] = { ...s, text: e.target.value }; setItems(n); }}
            onBlur={() => persist(items)} />
          <Button onClick={() => persist(items.filter((_, j) => j !== i))}>delete</Button>
        </div>
      ))}
      <div className="flex gap-2">
        <Button onClick={() => persist([...items, { keyword: "", text: "" }])}>+ add</Button>
        <Button onClick={onClose}>close (Esc)</Button>
      </div>
      </div>
    </Window>
  );
}
