import { InputHTMLAttributes } from "react";

export function Input({ className = "", ...props }: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      {...props}
      className={
        "bg-white/6 border border-hair rounded-[7px] text-fg px-2.5 py-1.5 " +
        "text-[13px] outline-none focus:border-accent " +
        className
      }
    />
  );
}
