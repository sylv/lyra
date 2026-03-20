import type { ButtonHTMLAttributes, FC } from "react";
import { cn } from "../../../lib/utils";

export const PlayerButton: FC<ButtonHTMLAttributes<HTMLButtonElement>> = ({ children, className, ...props }) => {
	return (
		<button
			type="button"
			className={cn(
				"p-3 rounded transition-colors text-white hover:bg-zinc-600/30 hover:backdrop-blur-md disabled:opacity-45 disabled:cursor-not-allowed disabled:hover:bg-transparent disabled:hover:backdrop-blur-none",
				className,
			)}
			{...props}
		>
			{children}
		</button>
	);
};
