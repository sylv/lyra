import type { FC } from "react";
import { PlayerDialog } from "./player-dialog";

interface PlayerPickDialogProps {
  options: { id: string; label: string }[];
  onSelect: (id: string) => void;
}

export const PlayerPickDialog: FC<PlayerPickDialogProps> = ({ options, onSelect }) => {
  return (
    <PlayerDialog>
      <div className="bg-zinc-900 rounded">
        {options.map((option) => (
          <button
            key={option.id}
            onClick={() => onSelect(option.id)}
            className="block w-full text-left px-6 py-4 hover:bg-zinc-800 transition duration-75"
          >
            {option.label}
          </button>
        ))}
      </div>
    </PlayerDialog>
  );
};
