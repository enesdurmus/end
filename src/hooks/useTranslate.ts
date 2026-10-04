import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Meaning, TranslationEntry } from "../types";
import { Mode } from "../lib/navigation";

type Raw = { text: string; detected: string; alternatives: string[]; meanings: Meaning[] };

// ponytail: same shape as useFileSearch — debounce + cancel flag, no request library
export function useTranslate(mode: Mode, query: string, source: string, target: string) {
  const [entry, setEntry] = useState<TranslationEntry | null>(null);
  const [alternatives, setAlternatives] = useState<string[]>([]);
  const [meanings, setMeanings] = useState<Meaning[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const text = mode === "translate" ? query.trim() : "";

  useEffect(() => {
    if (!text) { setEntry(null); setAlternatives([]); setMeanings([]); setError(""); setLoading(false); return; }
    let cancelled = false;
    setLoading(true);
    setError("");
    const t = setTimeout(() => {
      // "auto" means "let the backend detect"; only a pinned source is sent
      invoke<Raw>("translate", { text, from: source === "auto" ? null : source, to: target })
        .then((r) => {
          if (cancelled) return;
          setEntry({ source: text, translated: r.text, from: r.detected, to: target });
          setAlternatives(r.alternatives);
          setMeanings(r.meanings);
          setError("");
        })
        .catch((e) => {
          if (cancelled) return;
          setEntry(null);
          setAlternatives([]);
          setMeanings([]);
          setError(String(e));
        })
        .finally(() => { if (!cancelled) setLoading(false); });
    }, 300);
    return () => { cancelled = true; clearTimeout(t); };
  }, [text, source, target]);

  return { entry, alternatives, meanings, loading, error };
}
