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
        "inline-flex items-center justify-center gap-2 rounded-md px-3 py-1.5 text-[12px] cursor-pointer transition-colors " +
        (variant === "primary"
          ? "surface-active text-white hover:brightness-110 "
          : "surface-chip hover:text-fg hover:bg-[#aab4d6]/20 ") +
        className
      }
    />
  );
}
