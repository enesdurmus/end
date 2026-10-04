import { ReactNode } from "react";

export function Window({ className = "", children }: { className?: string; children: ReactNode }) {
  // The panel is the OS window minus this gutter, which is just room for the drop
  // shadow. Size lives in tauri.conf.json only; everything inside is relative to it.
  return (
    <div className="h-screen p-10">
      <div className={`flex flex-col h-full text-fg overflow-hidden rounded-window window-glass ${className}`}>
        {children}
      </div>
    </div>
  );
}
