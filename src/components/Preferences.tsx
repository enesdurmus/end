import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "./ui/Window";
import { Button } from "./ui/Button";
import { Field } from "./ui/Field";
import { Input } from "./ui/Input";
import { Select } from "./ui/Select";
import { Icon } from "./ui/Icon";
import { StatusBar } from "./StatusBar";
import { LANGUAGES } from "../lib/languages";

type Prefs = {
  toggle_shortcut: string;
  clipboard_shortcut: string;
  history_limit: number;
  image_limit: number;
  translate_target: string;
  translate_provider: string;
  klipy_api_key: string;
  gif_dir: string;
};
type Kind = "toggle" | "clipboard";

const MODIFIER_CODES = new Set(["MetaLeft", "MetaRight", "OSLeft", "OSRight", "ControlLeft", "ControlRight", "AltLeft", "AltRight", "ShiftLeft", "ShiftRight"]);

// WebKitGTK can report altKey/ctrlKey/... as false (or stale) while the key is
// held, so also trust the modifier keydown/keyup events we saw ourselves.
function acceleratorFromEvent(e: KeyboardEvent, held: Set<string>): string | null {
  if (MODIFIER_CODES.has(e.code)) return null;
  const has = (flag: boolean, ...codes: string[]) => flag || codes.some((c) => held.has(c));
  const parts: string[] = [];
  if (has(e.ctrlKey, "ControlLeft", "ControlRight")) parts.push("Control");
  if (has(e.altKey, "AltLeft", "AltRight")) parts.push("Alt");
  if (has(e.shiftKey, "ShiftLeft", "ShiftRight")) parts.push("Shift");
  if (has(e.metaKey, "MetaLeft", "MetaRight", "OSLeft", "OSRight")) parts.push("Super");
  if (parts.length === 0) return null;
  parts.push(e.code);
  return parts.join("+");
}

function HotkeyRow({ label, kind, value, onChanged }: { label: string; kind: Kind; value: string; onChanged: (accel: string) => void }) {
  const [recording, setRecording] = useState(false);
  const [error, setError] = useState("");

  // The current global shortcuts must not fire (or swallow the keys) while recording.
  useEffect(() => {
    if (!recording) return;
    const stop = () => setRecording(false);
    invoke("pause_shortcuts");
    window.addEventListener("blur", stop);
    return () => {
      window.removeEventListener("blur", stop);
      invoke("resume_shortcuts");
    };
  }, [recording]);

  useEffect(() => {
    if (!recording) return;
    const held = new Set<string>();
    const onUp = (e: KeyboardEvent) => held.delete(e.code);
    const onBlur = () => held.clear();
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      if (MODIFIER_CODES.has(e.code)) held.add(e.code);
      if (e.key === "Escape") { e.stopPropagation(); setRecording(false); return; }
      const accel = acceleratorFromEvent(e, held);
      if (!accel) return;
      setRecording(false);
      // Space's keyup would otherwise "click" the still-focused button, restart
      // the recording and wipe the error below.
      (document.activeElement as HTMLElement | null)?.blur();
      invoke("set_shortcut", { kind, accelerator: accel })
        .then(() => { setError(""); onChanged(accel); })
        .catch((err) => setError(String(err)));
    };
    window.addEventListener("keydown", onKey, true);
    window.addEventListener("keyup", onUp, true);
    window.addEventListener("blur", onBlur);
    return () => {
      window.removeEventListener("keydown", onKey, true);
      window.removeEventListener("keyup", onUp, true);
      window.removeEventListener("blur", onBlur);
    };
  }, [recording, kind, onChanged]);

  return (
    <Field label={label} error={error}>
      <Button className="min-w-[180px]" onClick={() => { setRecording(true); setError(""); }}>
        {recording ? "Waiting for keys…" : value.replace(/\+/g, " + ")}
      </Button>
    </Field>
  );
}

type LimitRow = {
  label: string;
  command: string;
  min: number;
  max: number;
  hint?: string;
  value: number;
  onChanged: (n: number) => void;
};

// Images are capped apart from text: one screenshot costs what a thousand
// snippets do. The backend clamps to the same range; mirroring it here just
// keeps the field showing the value that was actually persisted.
function LimitRow({ label, command, min, max, hint, value, onChanged }: LimitRow) {
  const [text, setText] = useState(String(value));
  const [error, setError] = useState("");

  const commit = () => {
    const n = Number(text);
    if (!Number.isFinite(n) || n < min) { setError(`Must be a number ≥ ${min}`); return; }
    const limit = Math.min(Math.max(Math.trunc(n), min), max);
    invoke(command, { limit })
      .then(() => { setError(""); setText(String(limit)); onChanged(limit); })
      .catch((err) => setError(String(err)));
  };

  return (
    <Field label={label} hint={hint} error={error}>
      <Input
        type="number" min={min} max={max} className="w-24" value={text}
        onChange={(e) => setText(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => { if (e.key === "Enter") (e.target as HTMLInputElement).blur(); }}
      />
    </Field>
  );
}

function TranslateRows({ prefs, onChanged }: { prefs: Prefs; onChanged: (p: Partial<Prefs>) => void }) {
  const [error, setError] = useState("");

  // send only what changed; the backend leaves the omitted field alone
  const commit = (next: Partial<Prefs>) => {
    invoke("set_translate_prefs", {
      target: next.translate_target ?? null,
      provider: next.translate_provider ?? null,
    })
      .then(() => { setError(""); onChanged(next); })
      .catch((err) => setError(String(err)));
  };

  return (
    <>
      <Field label="Translate To">
        <Select
          value={prefs.translate_target}
          options={LANGUAGES.map((l) => ({ value: l.code, label: l.name }))}
          onChange={(v) => commit({ translate_target: v })}
        />
      </Field>
      <Field label="Translation Engine" error={error}>
        <Select
          value={prefs.translate_provider}
          // mirrors the Provider enum in src-tauri/src/translate.rs
          options={[{ value: "google", label: "Google" }]}
          onChange={(v) => commit({ translate_provider: v })}
        />
      </Field>
    </>
  );
}

function GifRows({ prefs, onChanged }: { prefs: Prefs; onChanged: (p: Partial<Prefs>) => void }) {
  const [key, setKey] = useState(prefs.klipy_api_key);
  const [dir, setDir] = useState(prefs.gif_dir);
  const [error, setError] = useState("");

  const commit = (next: Partial<Prefs>) => {
    invoke("set_gif_prefs", {
      klipyApiKey: next.klipy_api_key ?? null,
      gifDir: next.gif_dir ?? null,
    })
      .then(() => { setError(""); onChanged(next); })
      .catch((err) => setError(String(err)));
  };

  return (
    <>
      <Field label="KLIPY API Key" hint="Boş bırakırsan uygulamanın kendi anahtarı kullanılır.">
        <Input value={key} onChange={(e) => setKey(e.target.value)} onBlur={() => commit({ klipy_api_key: key })} />
      </Field>
      <Field label="GIF Folder" hint="Boş bırakırsan varsayılan klasör kullanılır." error={error}>
        <Input value={dir} onChange={(e) => setDir(e.target.value)} onBlur={() => commit({ gif_dir: dir })} />
      </Field>
    </>
  );
}

export function Preferences({ onClose }: { onClose: () => void }) {
  const [prefs, setPrefs] = useState<Prefs | null>(null);
  const [accessible, setAccessible] = useState<boolean | null>(null);

  const recheck = () => invoke<boolean>("check_accessibility").then(setAccessible);

  useEffect(() => {
    invoke<Prefs>("get_preferences").then(setPrefs);
    recheck();
  }, []);

  if (!prefs) return null;

  return (
    <Window>
      <div className="scroll-thin flex-1 min-h-0 overflow-y-auto p-5">
      <div className="flex items-center gap-3 mb-5">
        <Button className="h-8 w-8 p-0!" aria-label="Back" onClick={onClose}><Icon name="left" size={16} /></Button>
        <h3 className="m-0 text-[17px] font-medium">Preferences</h3>
      </div>

      <div className="grid grid-cols-2 gap-x-6">
      <HotkeyRow label="Toggle End" kind="toggle" value={prefs.toggle_shortcut}
        onChanged={(accel) => setPrefs({ ...prefs, toggle_shortcut: accel })} />
      <HotkeyRow label="Clipboard History" kind="clipboard" value={prefs.clipboard_shortcut}
        onChanged={(accel) => setPrefs({ ...prefs, clipboard_shortcut: accel })} />

      <LimitRow label="Clipboard History Limit" command="set_history_limit" min={1} max={10000}
        value={prefs.history_limit}
        onChanged={(n) => setPrefs({ ...prefs, history_limit: n })} />

      <LimitRow label="Clipboard Image Limit" command="set_image_limit" min={0} max={1000}
        hint="Images are kept separately; 0 keeps none."
        value={prefs.image_limit}
        onChanged={(n) => setPrefs({ ...prefs, image_limit: n })} />

      <TranslateRows prefs={prefs} onChanged={(p) => setPrefs({ ...prefs, ...p })} />

      <GifRows prefs={prefs} onChanged={(p) => setPrefs({ ...prefs, ...p })} />
      </div>

      <div className="mt-5 pt-4">
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
      </div>
      <StatusBar left="" hints={[["Back", "esc"]]} />
    </Window>
  );
}
