import { Check } from "lucide-react";
import type { FC, ReactNode } from "react";
import { cn } from "../../lib/utils";

interface CheckboxCardProps {
  id: string;
  checked: boolean;
  disabled?: boolean;
  title: ReactNode;
  description?: ReactNode;
  onCheckedChange: (checked: boolean) => void;
  className?: string;
}

export const CheckboxCard: FC<CheckboxCardProps> = ({
  id,
  checked,
  disabled = false,
  title,
  description,
  onCheckedChange,
  className,
}) => (
  <label
    htmlFor={id}
    className={cn(
      "flex w-full items-start gap-3 rounded border border-zinc-800 bg-zinc-950/60 px-3 py-3 text-left transition-colors",
      disabled ? "cursor-not-allowed opacity-60" : "cursor-pointer hover:border-zinc-700 hover:bg-zinc-950/80",
      className,
    )}
  >
    <input
      id={id}
      type="checkbox"
      className="sr-only"
      checked={checked}
      disabled={disabled}
      onChange={(event) => onCheckedChange(event.target.checked)}
    />
    <div
      aria-hidden
      className={cn(
        "mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-[4px] border shadow-xs",
        checked ? "border-emerald-400 bg-emerald-400 text-black" : "border-zinc-700 bg-zinc-900",
      )}
    >
      {checked ? <Check className="size-3" /> : null}
    </div>
    <div>
      <div className="text-sm font-medium text-zinc-100">{title}</div>
      {description ? <p className="mt-1 text-sm text-zinc-400">{description}</p> : null}
    </div>
  </label>
);
