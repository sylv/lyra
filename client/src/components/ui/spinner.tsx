import type { ComponentProps, FC } from "react";
import { Loader2Icon } from "lucide-react";

import { cn } from "@/lib/utils";

export const Spinner: FC<ComponentProps<"svg">> = ({ className, ...props }) => {
  return <Loader2Icon role="status" aria-label="Loading" className={cn("size-4 animate-spin", className)} {...props} />;
};
