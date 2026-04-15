import type { FC } from "react";
import { generateGradientIcon } from "../lib/generate-gradient-icon";
import { cn } from "../lib/utils";

interface UserAvatarProps {
  createdAt: number;
  alt?: string;
  className?: string;
  size?: number;
}

export const UserAvatar: FC<UserAvatarProps> = ({ createdAt, alt = "", className, size = 32 }) => {
  const icon = generateGradientIcon(createdAt.toString(), { size });

  return <img src={icon} alt={alt} className={cn("rounded-full", className)} />;
};
