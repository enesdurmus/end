import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Snippet = { keyword: string; text: string };

export function SnippetManager({ onClose }: { onClose: () => void }) {
  const [items, setItems] = useState<Snippet[]>([]);
  useEffect(() => { invoke<Snippet[]>("list_snippets").then(setItems); }, []);
  const persist = (next: Snippet[]) => { setItems(next); invoke("save_snippets", { items: next }); };
  return (
    <div className="app" style={{ padding: 16 }}>
      <h3>Snippets</h3>
      {items.map((s, i) => (
        <div key={i} style={{ display: "flex", gap: 8, marginBottom: 6 }}>
          <input value={s.keyword} placeholder="keyword"
            onChange={(e) => { const n = [...items]; n[i] = { ...s, keyword: e.target.value }; setItems(n); }}
            onBlur={() => persist(items)} />
          <input value={s.text} placeholder="metin" style={{ flex: 1 }}
            onChange={(e) => { const n = [...items]; n[i] = { ...s, text: e.target.value }; setItems(n); }}
            onBlur={() => persist(items)} />
          <button onClick={() => persist(items.filter((_, j) => j !== i))}>sil</button>
        </div>
      ))}
      <button onClick={() => persist([...items, { keyword: "", text: "" }])}>+ ekle</button>
      <button onClick={onClose} style={{ marginLeft: 8 }}>kapat (Esc)</button>
    </div>
  );
}
