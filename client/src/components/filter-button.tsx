import type { FC, ReactNode } from "react";
import { cn } from "../lib/utils";

interface FilterButtonProps {
	children: ReactNode;
	active?: boolean;
	onClick: (event: React.MouseEvent<HTMLButtonElement>) => void;
}

export const FilterButton: FC<FilterButtonProps> = ({
	children,
	onClick,
	active = false,
}) => {
	return (
		<button
			className={cn(`flex rounded-lg px-4 py-0.5 text-sm gap-2 items-center transition-colors border border-zinc-700/50 text-zinc-200`, active ? 'bg-zinc-200/15' : 'hover:bg-zinc-200/10')}
			type="button"
			onClick={onClick}
		>
			{children}
		</button>
	);
};

// todo: "FilterSelect" component that drops down and, if an option is selected, expands to show an "X" on the right side to remove it.
// the "FilterSelect" would have an icon on the left and a chevron on the right.
// the expanded section would be a darker colour (solid black?) compared to the "main" body, which would resemble a FilterButton.
