import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "./ui/Window";
import { Button } from "./ui/Button";
import { Field } from "./ui/Field";
import { Input } from "./ui/Input";

type Prefs = { toggle_shortcut: string; clipboard_shortcut: string; history_limit: number };
type Kind = "toggle" | "clipboard";

const MODIFIER_CODES = new Set(["MetaLeft", "MetaRight", "ControlLeft", "ControlRight", "AltLeft", "AltRight", "ShiftLeft", "ShiftRight"]);

function acceleratorFromEvent(e: KeyboardEvent): string | null {
  if (MODIFIER_CODES.has(e.code)) return null;
  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Control");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Super");
  if (parts.length === 0) return null;
  parts.push(e.code);
  return parts.join("+");
}

function HotkeyRow({ label, kind, value, onChanged }: { label: string; kind: Kind; value: string; onChanged: (accel: string) => void }) {
  const [recording, setRecording] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    if (!recording) return;
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      if (e.key === "Escape") { setRecording(false); return; }
      const accel = acceleratorFromEvent(e);
      if (!accel) return;
      setRecording(false);
      invoke("set_shortcut", { kind, accelerator: accel })
        .then(() => { setError(""); onChanged(accel); })
        .catch((err) => setError(String(err)));
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [recording, kind, onChanged]);

  return (
    <Field label={label}>
      <Button className="min-w-[180px]" onClick={() => { setRecording(true); setError(""); }}>
        {recording ? "Waiting for keys…" : value.replace(/\+/g, " + ")}
      </Button>
      {error && <div className="text-danger text-xs mt-1">{error}</div>}
    </Field>
  );
}

// backend clamps to this range too; mirror it so the UI shows the persisted value
const clampLimit = (n: number) => Math.min(Math.max(Math.trunc(n), 1), 10000);

function HistoryLimitRow({ value, onChanged }: { value: number; onChanged: (n: number) => void }) {
  const [text, setText] = useState(String(value));
  const [error, setError] = useState("");

  const commit = () => {
    const n = Number(text);
    if (!Number.isFinite(n) || n < 1) { setError("Must be a number ≥ 1"); return; }
    const limit = clampLimit(n);
    invoke("set_history_limit", { limit })
      .then(() => { setError(""); setText(String(limit)); onChanged(limit); })
      .catch((err) => setError(String(err)));
  };

  return (
    <Field label="Clipboard History Limit">
      <Input
        type="number" min={1} max={10000} className="w-24" value={text}
        onChange={(e) => setText(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => { if (e.key === "Enter") (e.target as HTMLInputElement).blur(); }}
      />
      {error && <div className="text-danger text-xs mt-1">{error}</div>}
    </Field>
  );
}

export function Preferences() {
  const [prefs, setPrefs] = useState<Prefs | null>(null);
  const [accessible, setAccessible] = useState<boolean | null>(null);

  const recheck = () => invoke<boolean>("check_accessibility").then(setAccessible);

  useEffect(() => {
    invoke<Prefs>("get_preferences").then(setPrefs);
    recheck();
  }, []);

  if (!prefs) return null;

  return (
    <Window variant="flat" className="p-5">
      <h3 className="m-0 mb-4 text-[15px] font-semibold">Preferences</h3>

      <HotkeyRow label="Toggle Launcher" kind="toggle" value={prefs.toggle_shortcut}
        onChanged={(accel) => setPrefs({ ...prefs, toggle_shortcut: accel })} />
      <HotkeyRow label="Clipboard History" kind="clipboard" value={prefs.clipboard_shortcut}
        onChanged={(accel) => setPrefs({ ...prefs, clipboard_shortcut: accel })} />

      <HistoryLimitRow value={prefs.history_limit}
        onChanged={(n) => setPrefs({ ...prefs, history_limit: n })} />

      <div className="mt-5 pt-4 border-t border-hair">
        <div className="text-[13px] mb-2">
          Accessibility permission:{" "}
          <span className={accessible ? "text-success" : "text-danger"}>
            {accessible === null ? "checking…" : accessible ? "Granted" : "Not granted"}
          </span>
        </div>
        <div className="flex gap-2">
          <Button onClick={recheck}>Re-check</Button>
          {!accessible && (
            <Button onClick={() => invoke("open_accessibility_settings")}>Open System Settings</Button>
          )}
        </div>
      </div>
    </Window>
  );
}
