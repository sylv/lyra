import type { FC } from "react";
import { motion } from "framer-motion";

interface PlayerDialogProps {
  children: React.ReactNode;
}

export const PlayerDialog: FC<PlayerDialogProps> = ({ children }) => {
  return (
    <motion.div
      className="fixed inset-0 bg-black/30 backdrop-blur-sm text-sm font-semibold flex items-center justify-center z-100"
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.95 }}
    >
      {children}
    </motion.div>
  );
};
