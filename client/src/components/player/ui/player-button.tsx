import type { ButtonHTMLAttributes, FC } from "react";
import { cn } from "../../../lib/utils";

export const PlayerButton: FC<ButtonHTMLAttributes<HTMLButtonElement>> = ({ children, className, ...props }) => {
  return (
    <button
      type="button"
      className={cn(
        "inline-flex h-10 w-10 items-center justify-center rounded-full text-white transition-colors hover:bg-white/12 focus-visible:bg-white/12 disabled:cursor-not-allowed disabled:text-white/40 disabled:hover:bg-transparent",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
};
