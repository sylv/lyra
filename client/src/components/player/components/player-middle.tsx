import type { FC, ReactNode } from "react";

export const PlayerMiddle: FC<{ children: ReactNode }> = ({ children }) => {
  return <div className="absolute inset-0">{children}</div>;
};
