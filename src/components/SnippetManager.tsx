import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Snippet = { keyword: string; text: string };

export function SnippetManager({ onClose }: { onClose: () => void }) {
  const [items, setItems] = useState<Snippet[]>([]);
  useEffect(() => { invoke<Snippet[]>("list_snippets").then(setItems); }, []);
  const persist = (next: Snippet[]) => { setItems(next); invoke("save_snippets", { items: next }); };
  return (
    <div className="app panel">
      <h3>Snippets</h3>
      {items.map((s, i) => (
        <div key={i} style={{ display: "flex", gap: 8, marginBottom: 8 }}>
          <input className="input" value={s.keyword} placeholder="keyword"
            onChange={(e) => { const n = [...items]; n[i] = { ...s, keyword: e.target.value }; setItems(n); }}
            onBlur={() => persist(items)} />
          <input className="input" value={s.text} placeholder="text" style={{ flex: 1 }}
            onChange={(e) => { const n = [...items]; n[i] = { ...s, text: e.target.value }; setItems(n); }}
            onBlur={() => persist(items)} />
          <button className="btn" onClick={() => persist(items.filter((_, j) => j !== i))}>delete</button>
        </div>
      ))}
      <button className="btn" onClick={() => persist([...items, { keyword: "", text: "" }])}>+ add</button>
      <button className="btn" onClick={onClose} style={{ marginLeft: 8 }}>close (Esc)</button>
    </div>
  );
}
