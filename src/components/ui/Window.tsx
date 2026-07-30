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
        "flex flex-col h-screen text-fg bg-bg " +
        "border border-hair font-[-apple-system,BlinkMacSystemFont,'SF_Pro_Text',sans-serif] " +
        (variant === "floating"
          ? "overflow-hidden rounded-window backdrop-blur-window "
          : "overflow-y-auto ") +
        className
      }
    >
      {children}
    </div>
  );
}
