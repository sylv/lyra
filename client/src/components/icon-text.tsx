import type { FC, ReactNode } from "react";
import { cn } from "../lib/utils";

type IconTextProps = {
	icon: ReactNode;
	text: string;
	className?: string;
};

export const IconText: FC<IconTextProps> = ({ icon, text, className }) => {
	return (
		<div
			className={cn("flex flex-col items-center justify-center h-full min-h-full select-none text-zinc-300", className)}
		>
			{icon}
			<div className="m-2 text-xs font-semibold max-w-80 text-center">{text}</div>
		</div>
	);
};
