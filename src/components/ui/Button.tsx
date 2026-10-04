import { ButtonHTMLAttributes } from "react";

export function Button({ className = "", ...props }: ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      {...props}
      className={
        "surface-chip rounded-xl px-3.5 py-2 text-[13px] cursor-pointer transition-colors " +
        "hover:text-fg hover:border-[#7c4dff]/60 hover:bg-[#4630b9]/40 " +
        className
      }
    />
  );
}
