import { Plus } from "lucide-react";
import type { FC, ReactNode } from "react";
import { cn } from "../../lib/utils";
import { Spinner } from "../ui/spinner";

const cardClassName = "min-h-36 rounded border border-zinc-700 p-4 text-zinc-100";
const cardIconClassName = "flex size-10 shrink-0 items-center justify-center rounded bg-zinc-700/80";

interface ManagementCardProps {
  icon: ReactNode;
  title: ReactNode;
  subtitle?: ReactNode;
  subtitleClassName?: string;
  actions?: ReactNode;
  children?: ReactNode;
  footer?: ReactNode;
  className?: string;
}

export const ManagementCard: FC<ManagementCardProps> = ({
  icon,
  title,
  subtitle,
  subtitleClassName,
  actions,
  children,
  footer,
  className,
}) => (
  <div className={cn("flex flex-col justify-between", cardClassName, className)}>
    <div>
      <div className="flex items-start justify-between gap-3">
        <div className="flex min-w-0 items-center gap-4">
          <div className={cardIconClassName}>{icon}</div>
          <div className="min-w-0">
            <h3 className="truncate font-medium">{title}</h3>
            {subtitle ? <p className={cn("mt-0.5 text-xs text-zinc-400", subtitleClassName)}>{subtitle}</p> : null}
          </div>
        </div>
        {actions}
      </div>
      {children ? <div className="mt-4 space-y-3">{children}</div> : null}
    </div>
    {footer ? <div className="mt-4 text-xs text-zinc-500">{footer}</div> : null}
  </div>
);

interface ManagementCreateCardProps {
  title: ReactNode;
  description: ReactNode;
  onClick: () => void;
  loading?: boolean;
  className?: string;
}

export const ManagementCreateCard: FC<ManagementCreateCardProps> = ({
  title,
  description,
  onClick,
  loading = false,
  className,
}) => (
  <button
    type="button"
    onClick={onClick}
    disabled={loading}
    className={cn(
      cardClassName,
      "flex w-full flex-col justify-between text-left opacity-80 hover:opacity-100 transition-opacity group",
      "disabled:cursor-wait disabled:opacity-80",
      className,
    )}
  >
    {loading ? (
      <div className="flex min-h-28 flex-1 items-center justify-center">
        <Spinner />
      </div>
    ) : (
      <div className="flex items-start gap-4">
        <div className={cn(cardIconClassName, "bg-transparent")}>
          <Plus className="size-5" />
        </div>
        <div className="min-w-0">
          <h3 className="font-medium group-hover:underline">{title}</h3>
          <p className="mt-1 text-xs text-zinc-400">{description}</p>
        </div>
      </div>
    )}
  </button>
);
