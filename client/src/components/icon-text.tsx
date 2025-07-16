import type { FC, ReactNode } from "react";

type IconTextProps = {
	icon: ReactNode;
	text: string;
};

export const IconText: FC<IconTextProps> = ({ icon, text }) => {
	return (
		<div className="flex flex-col items-center justify-center h-full min-h-full select-none">
			{icon}
			<div className="m-2 font-mono text-zinc-300 uppercase text-xs max-w-[240px] text-center">
				{text}
			</div>
		</div>
	);
};
