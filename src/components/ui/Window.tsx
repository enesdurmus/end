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
  return (
    <div
      className={
        "flex flex-col h-screen overflow-hidden text-fg bg-bg backdrop-blur-window " +
        "border border-hair font-[-apple-system,BlinkMacSystemFont,'SF_Pro_Text',sans-serif] " +
        (variant === "floating" ? "rounded-window " : "") +
        className
      }
    >
      {children}
    </div>
  );
}
