import type { FC, ReactNode } from "react";

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
			className={`flex rounded-lg px-4 py-0.5 text-sm gap-2 items-center transition-colors ${
				active
					? "bg-indigo-600 text-white"
					: "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
			}`}
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
