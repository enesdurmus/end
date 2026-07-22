import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Prefs = { toggle_shortcut: string; clipboard_shortcut: string };
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
    <div style={{ marginBottom: 12 }}>
      <div style={{ fontSize: 13, marginBottom: 4 }}>{label}</div>
      <button className="btn" onClick={() => { setRecording(true); setError(""); }} style={{ minWidth: 180 }}>
        {recording ? "Waiting for keys…" : value.replace(/\+/g, " + ")}
      </button>
      {error && <div style={{ color: "#e03131", fontSize: 12, marginTop: 4 }}>{error}</div>}
    </div>
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
    <div className="app panel" style={{ borderRadius: 0 }}>
      <h3>Preferences</h3>

      <HotkeyRow label="Toggle Launcher" kind="toggle" value={prefs.toggle_shortcut}
        onChanged={(accel) => setPrefs({ ...prefs, toggle_shortcut: accel })} />
      <HotkeyRow label="Clipboard History" kind="clipboard" value={prefs.clipboard_shortcut}
        onChanged={(accel) => setPrefs({ ...prefs, clipboard_shortcut: accel })} />

      <div style={{ marginTop: 20, paddingTop: 16, borderTop: "1px solid var(--hair)" }}>
        <div style={{ fontSize: 13, marginBottom: 8 }}>
          Accessibility permission:{" "}
          <span style={{ color: accessible ? "#40c057" : "#e03131" }}>
            {accessible === null ? "checking…" : accessible ? "Granted" : "Not granted"}
          </span>
        </div>
        <button className="btn" onClick={recheck}>Re-check</button>
        {!accessible && (
          <button className="btn" style={{ marginLeft: 8 }} onClick={() => invoke("open_accessibility_settings")}>
            Open System Settings
          </button>
        )}
      </div>
    </div>
  );
}
