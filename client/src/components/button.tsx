import { Loader2, type LucideIcon } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import type { ButtonHTMLAttributes, FC, ReactNode } from "react";
import { cn } from "../lib/utils";

export enum ButtonStyle {
  Primary = "bg-amethyst-600/50 not-disabled:hover:bg-amethyst-600/70",
  Plex = "bg-[#e5a00d] text-black not-disabled:hover:bg-[#e5a00d]/90",
  Glass = "bg-zinc-700/30 text-zinc-200 not-disabled:hover:bg-zinc-700/50",
  White = "bg-zinc-100 text-zinc-950 not-disabled:hover:bg-white",
  Transparent = "bg-transparent text-zinc-400 not-disabled:hover:bg-zinc-500/20 not-disabled:hover:text-zinc-100",
}

export enum ButtonSize {
  Smol = "px-3 py-1.5 gap-2 text-xs",
  Normal = "px-6 py-2 gap-3 text-normal",
}

interface ButtonProps {
  children: ReactNode;
  icon?: [string, LucideIcon];
  iconSide?: "left" | "right";
  loading?: boolean;
  className?: string;
  style?: ButtonStyle;
  size?: ButtonSize;
  disabled?: boolean;
  onClick?: () => void;
  type?: ButtonHTMLAttributes<HTMLButtonElement>["type"];
}

export const Button: FC<ButtonProps> = ({
  children,
  className,
  icon,
  iconSide = "right",
  loading,
  style = ButtonStyle.Primary,
  size = ButtonSize.Normal,
  disabled,
  onClick,
  type = "button",
}) => {
  if (loading) {
    icon = ["loading-spinner", Loader2];
  }

  const [iconKey, Icon] = icon ?? ["no-icon", null];
  const iconTranslate = iconSide === "right" ? "group-hover:translate-x-0.5" : "group-hover:-translate-x-0.5";
  const iconElement = Icon ? (
    <div
      className={cn(
        "h-4 w-4 flex items-center justify-center relative overflow-hidden",
        !loading && !disabled && `transition-all ${iconTranslate}`,
      )}
    >
      <AnimatePresence initial={false} mode="wait">
        <motion.div
          key={iconKey}
          initial={{ y: 20, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          exit={{ y: -20, opacity: 0 }}
          className="flex items-center justify-center absolute inset-0"
          transition={{
            type: "spring",
            stiffness: 200,
            damping: 25,
            duration: 0.175,
          }}
        >
          <Icon className={cn("h-4 w-4", loading && "animate-spin")} />
        </motion.div>
      </AnimatePresence>
    </div>
  ) : null;

  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled || loading}
      className={cn(
        "rounded-sm flex items-center font-semibold text-sm transition-colors group not-disabled:hover:underline",
        "disabled:opacity-80 disabled:cursor-not-allowed",
        size,
        style,
        className,
      )}
    >
      {iconSide === "left" && iconElement}
      {children}
      {iconSide === "right" && iconElement}
    </button>
  );
};
