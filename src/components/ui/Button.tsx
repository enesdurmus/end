import { ButtonHTMLAttributes } from "react";

export function Button({
  variant,
  className = "",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: "primary" }) {
  return (
    <button
      {...props}
      className={
        "inline-flex items-center justify-center gap-2 rounded-[10px] px-3.5 py-2 text-[13px] cursor-pointer transition-colors " +
        (variant === "primary"
          ? "surface-active text-white hover:brightness-110 "
          : "surface-chip hover:text-fg hover:bg-[#7d78ff]/20 ") +
        className
      }
    />
  );
}
