import { ReactNode, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Window } from "./ui/Window";
import { Button } from "./ui/Button";
import { Input } from "./ui/Input";
import { Kbd } from "./ui/Kbd";
import { Select } from "./ui/Select";
import { Icon, IconName } from "./ui/Icon";
import { StatusBar } from "./StatusBar";
import { LANGUAGES } from "../lib/languages";

type Prefs = {
  toggle_shortcut: string;
  clipboard_shortcut: string;
  screenshot_shortcut: string;
  window_management: boolean;
  history_limit: number;
  image_limit: number;
  translate_target: string;
  gif_dir: string;
};
type Kind = "toggle" | "clipboard" | "screenshot";

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

const keyParts = (accel: string) =>
  accel.split("+").map((k) => k.replace(/^(Key|Digit)/, "").replace(/^Arrow/, ""));

function PrefRow({ label, hint, error, children }: { label: string; hint?: string; error?: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-6 px-3.5 py-2.5 min-h-[50px] border-t border-hair first:border-t-0">
      <div className="min-w-0">
        <div className="text-[12.5px]">{label}</div>
        {error ? <div className="text-danger text-[11px] mt-0.5">{error}</div>
          : hint && <div className="text-fg-dim text-[11px] mt-0.5">{hint}</div>}
      </div>
      <div className="flex-none">{children}</div>
    </div>
  );
}

function Section({ title, icon, children }: { title: string; icon: IconName; children: ReactNode }) {
  return (
    <section className="mb-4">
      <div className="flex items-center gap-1.5 mb-1.5 px-1 text-[10.5px] uppercase tracking-wider text-fg-dim">
        <Icon name={icon} size={12} />{title}
      </div>
      <div className="card">{children}</div>
    </section>
  );
}

function Switch({ on, onToggle, label }: { on: boolean; onToggle: () => void; label: string }) {
  return (
    <button
      role="switch" aria-checked={on} aria-label={label} onClick={onToggle}
      className={
        "relative block w-[34px] h-[20px] rounded-full cursor-pointer transition-colors " +
        (on ? "surface-active" : "surface-field")
      }
    >
      <span className={
        "absolute top-[3px] left-[3px] w-[14px] h-[14px] rounded-full bg-white shadow transition-transform " +
        (on ? "translate-x-[14px]" : "opacity-60")
      } />
    </button>
  );
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
    <PrefRow label={label} error={error}>
      <button
        onClick={() => { setRecording(true); setError(""); }}
        title="Click to record a new shortcut"
        className={
          "inline-flex items-center gap-1 h-[28px] px-2 rounded-md cursor-pointer transition-colors " +
          (recording ? "surface-active text-[11.5px]" : "hover:bg-[#aab4d6]/10")
        }
      >
        {recording ? "Press keys…" : keyParts(value).map((k) => <Kbd key={k}>{k}</Kbd>)}
      </button>
    </PrefRow>
  );
}

function SnapRow({ enabled, onChanged }: { enabled: boolean; onChanged: (on: boolean) => void }) {
  const [note, setNote] = useState("");

  // setup is idempotent; re-running it also picks up a finished re-login
  useEffect(() => {
    if (enabled) invoke<string>("window_management_note").then(setNote);
    else setNote("");
  }, [enabled]);

  const toggle = () =>
    invoke("set_window_management", { enabled: !enabled })
      .then(() => onChanged(!enabled))
      .catch((err) => setNote(String(err)));

  return (
    <PrefRow label="Window Snapping" hint="Ctrl+Alt + arrows, U I J K, D F G, E R T, Enter, C" error={note}>
      <Switch on={enabled} onToggle={toggle} label="Window Snapping" />
    </PrefRow>
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
    <PrefRow label={label} hint={hint} error={error}>
      <Input
        type="number" min={min} max={max} className="w-20 text-right" value={text}
        onChange={(e) => setText(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => { if (e.key === "Enter") (e.target as HTMLInputElement).blur(); }}
      />
    </PrefRow>
  );
}

function TranslateRow({ prefs, onChanged }: { prefs: Prefs; onChanged: (p: Partial<Prefs>) => void }) {
  const [error, setError] = useState("");

  const commit = (code: string) => {
    invoke("set_translate_prefs", { target: code, provider: null })
      .then(() => { setError(""); onChanged({ translate_target: code }); })
      .catch((err) => setError(String(err)));
  };

  return (
    <PrefRow label="Translate To" hint="Language text is translated into" error={error}>
      <Select
        className="w-[180px]"
        value={prefs.translate_target}
        options={LANGUAGES.map((l) => ({ value: l.code, label: l.name }))}
        onChange={commit}
      />
    </PrefRow>
  );
}

function GifRow({ prefs, onChanged }: { prefs: Prefs; onChanged: (p: Partial<Prefs>) => void }) {
  const [dir, setDir] = useState(prefs.gif_dir);
  const [error, setError] = useState("");

  const commit = () => {
    if (dir === prefs.gif_dir) return;
    invoke("set_gif_prefs", { klipyApiKey: null, gifDir: dir })
      .then(() => { setError(""); onChanged({ gif_dir: dir }); })
      .catch((err) => setError(String(err)));
  };

  return (
    <PrefRow label="GIF Folder" hint="Where saved GIFs are kept" error={error}>
      <div className="flex items-center gap-1.5">
        <Input
          className="w-[220px]" value={dir} placeholder="Default"
          onChange={(e) => setDir(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => { if (e.key === "Enter") (e.target as HTMLInputElement).blur(); }}
        />
        <Button className="h-[30px] w-[30px] p-0!" title="Open folder" aria-label="Open GIF folder"
          onClick={() => invoke("open_gif_dir")}>
          <Icon name="folder" size={14} />
        </Button>
      </div>
    </PrefRow>
  );
}

function AccessRow() {
  const [granted, setGranted] = useState<boolean | null>(null);
  const recheck = () => invoke<boolean>("check_accessibility").then(setGranted);
  useEffect(() => { recheck(); }, []);

  return (
    <PrefRow label="Accessibility" hint="Needed to paste into other apps">
      <div className="flex items-center gap-2">
        <span className="flex items-center gap-1.5 text-[11.5px] text-fg-dim">
          <span className={"w-1.5 h-1.5 rounded-full " + (granted ? "bg-success" : granted === false ? "bg-danger" : "bg-fg-dim")} />
          {granted === null ? "Checking…" : granted ? "Granted" : "Not granted"}
        </span>
        {granted === false && (
          <>
            <Button className="h-[26px]" onClick={recheck}>Re-check</Button>
            <Button className="h-[26px]" variant="primary" onClick={() => invoke("open_accessibility_settings")}>Open Settings</Button>
          </>
        )}
      </div>
    </PrefRow>
  );
}

export function Preferences({ onClose }: { onClose: () => void }) {
  const [prefs, setPrefs] = useState<Prefs | null>(null);

  useEffect(() => {
    invoke<Prefs>("get_preferences").then(setPrefs);
  }, []);

  if (!prefs) return null;
  const patch = (p: Partial<Prefs>) => setPrefs({ ...prefs, ...p });

  return (
    <Window>
      <div className="flex-none flex items-center gap-2.5 px-4 pt-3.5 pb-3">
        <Button className="h-7 w-7 p-0!" aria-label="Back" onClick={onClose}><Icon name="left" size={14} /></Button>
        <h3 className="m-0 text-[14px] font-medium">Preferences</h3>
      </div>

      <div className="scroll-thin flex-1 min-h-0 overflow-y-auto px-4 pb-2">
        <Section title="Shortcuts" icon="command">
          <HotkeyRow label="Toggle End" kind="toggle" value={prefs.toggle_shortcut}
            onChanged={(accel) => patch({ toggle_shortcut: accel })} />
          <HotkeyRow label="Clipboard History" kind="clipboard" value={prefs.clipboard_shortcut}
            onChanged={(accel) => patch({ clipboard_shortcut: accel })} />
          <HotkeyRow label="Screenshot" kind="screenshot" value={prefs.screenshot_shortcut}
            onChanged={(accel) => patch({ screenshot_shortcut: accel })} />
        </Section>

        <Section title="Clipboard" icon="clipboard">
          <LimitRow label="History Limit" command="set_history_limit" min={1} max={10000}
            hint="Text entries to keep"
            value={prefs.history_limit} onChanged={(n) => patch({ history_limit: n })} />
          <LimitRow label="Image Limit" command="set_image_limit" min={0} max={1000}
            hint="Kept separately; 0 keeps none"
            value={prefs.image_limit} onChanged={(n) => patch({ image_limit: n })} />
        </Section>

        <Section title="Translate & GIFs" icon="swap">
          <TranslateRow prefs={prefs} onChanged={patch} />
          <GifRow prefs={prefs} onChanged={patch} />
        </Section>

        <Section title="System" icon="settings">
          <SnapRow enabled={prefs.window_management} onChanged={(on) => patch({ window_management: on })} />
          <AccessRow />
        </Section>
      </div>
      <StatusBar left="" hints={[["Back", "esc"]]} />
    </Window>
  );
}
