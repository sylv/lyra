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

export const PaddedPlayerButton: FC<ButtonHTMLAttributes<HTMLButtonElement> & { side: "left" | "right" }> = ({
	children,
	side,
	className,
	...props
}) => {
	// todo: this is hacky. it makes it so moving your cursor to the bottom right of the window will activate
	// the fullscreen button, instead of having to move it to the bottom right then a liitlle up because of the padding.
	// its good UX, but hard coded values all over the place here suck.
	const classes = side === "left" ? "-ml-6 -mb-10 pl-6 pb-10 group/button" : "-mr-6 -mb-10 pr-6 pb-10 group/button";

	return (
		<button type="button" className={cn(classes, className)} {...props}>
			<div className="p-3 group-hover/button:bg-zinc-600/30 group-hover/button:backdrop-blur-md rounded transition-colors text-white">
				{children}
			</div>
		</button>
	);
};
