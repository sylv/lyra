import type { FC } from "react";
import { generateGradientIcon } from "../lib/generate-gradient-icon";
import { cn } from "../lib/utils";

interface LibraryIconProps {
  createdAt: number;
  alt?: string;
  className?: string;
  size?: number;
}

export const LibraryIcon: FC<LibraryIconProps> = ({ createdAt, alt = "", className, size = 32 }) => {
  const icon = generateGradientIcon(createdAt.toString(), { size });

  return <img src={icon} alt={alt} className={cn("rounded-md", className)} />;
};
