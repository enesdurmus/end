import { ButtonHTMLAttributes } from "react";

export function Button({ className = "", ...props }: ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      {...props}
      className={
        "bg-white/8 text-fg border border-hair rounded-[7px] px-3 py-1.5 " +
        "text-[13px] cursor-pointer hover:bg-white/15 " +
        className
      }
    />
  );
}
