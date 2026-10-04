import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { Result } from "../types";
import { runActions } from "../lib/actions";
import { formatBytes, KIND } from "../lib/results";
import { AppIcon } from "./ui/AppIcon";
import { Button } from "./ui/Button";
import { Icon, IconName } from "./ui/Icon";

// mirrors PathInfo in src-tauri/src/commands.rs (times are unix seconds)
type PathInfo = { size: number | null; is_dir: boolean; created: number | null; modified: number | null };

const date = (secs: number | null) =>
  secs == null ? "—" : new Date(secs * 1000).toLocaleString("en-GB", {
    day: "numeric", month: "short", year: "numeric", hour: "2-digit", minute: "2-digit",
  });

export function Detail({ result: r }: { result?: Result }) {
  const path = r && (r.type === "app" || r.type === "file") ? r.subtitle : undefined;
  const [info, setInfo] = useState<PathInfo | null>(null);

  // debounced: sweeping the pointer down the list must not stat every row it crosses
  useEffect(() => {
    setInfo(null);
    if (!path) return;
    let live = true;
    const t = setTimeout(() => {
      invoke<PathInfo | null>("path_info", { path }).then((i) => { if (live) setInfo(i ?? null); });
    }, 120);
    return () => { live = false; clearTimeout(t); };
  }, [path]);

  if (!r) return <div className="text-fg-dim text-[13px] grid place-items-center h-full">No selection</div>;

  const rows: [IconName, string, string][] = path
    ? [
        ["file", "Kind", info?.is_dir && r.type === "file" ? "Folder" : KIND[r.type]],
        ["plus", "Size", info?.size != null ? formatBytes(info.size) : "—"],
        ["calendar", "Created", date(info?.created ?? null)],
        ["clock", "Modified", date(info?.modified ?? null)],
        ["file", "Path", path],
      ]
    : [["file", "Kind", KIND[r.type]], ...(r.subtitle ? [["file", "Info", r.subtitle] as [IconName, string, string]] : [])];

  return (
    <div>
      <div className="flex items-center gap-4">
        <AppIcon title={r.title} src={r.icon} className="w-[52px] h-[52px]" />
        <div className="min-w-0">
          <div className="text-[17px] leading-tight truncate">{r.title}</div>
          <div className="text-[13px] mt-1 text-fg-dim">{KIND[r.type]}</div>
        </div>
      </div>

      <div className="flex gap-2.5 mt-5">
        <Button variant="primary" className="h-[38px]" onClick={() => r.run()}>
          <Icon name="play" size={15} />Open
        </Button>
        {path && (
          <>
            <Button className="h-[38px]"
              onClick={() => revealItemInDir(path).then(() => invoke("close_launcher"))}>
              <Icon name="folder" size={15} />Show in Folder
            </Button>
            <Button className="h-[38px]" onClick={() => runActions.copy(path)}>
              <Icon name="copy" size={15} />Copy Path
            </Button>
          </>
        )}
      </div>

      <div className="mt-6">
        <div className="text-sm text-[#c0c6dc] mb-2">Details</div>
        {rows.map(([icon, label, value]) => (
          <div key={label} className="grid grid-cols-[96px_1fr] items-start gap-3 py-[7px] text-[12.5px]">
            <span className="flex items-center gap-2.5 text-[#b4bbd3]"><Icon name={icon} size={15} />{label}</span>
            <span className="text-fg-dim break-all">{value}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
