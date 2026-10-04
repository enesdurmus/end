import { ReactNode } from "react";

export function Window({
  variant,
  className = "",
  children,
}: {
  variant: "floating" | "flat";
  className?: string;
  children: ReactNode;
}) {
  if (variant === "flat") {
    return (
      <div className={`flex flex-col h-screen text-fg window-fill border border-hair overflow-y-auto ${className}`}>
        {children}
      </div>
    );
  }
  // the transparent padding is room for the outer glow
  return (
    <div className="h-screen p-12">
      <div className={`flex flex-col h-full text-fg overflow-hidden rounded-window window-neon ${className}`}>
        {children}
      </div>
    </div>
  );
}
