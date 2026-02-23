import type { FC } from "react";

export const UnplayedItemsTab: FC<{ count: number }> = ({ count }) => {
	if (count === 0) return null;
	return (
		<div className="pointer-events-none absolute rounded-r-none right-0 top-2 inline-flex items-center justify-center rounded-full bg-black/80 pl-3 pr-2 text-xs font-semibold text-white backdrop-blur-sm">
			{count}
		</div>
	);
};
