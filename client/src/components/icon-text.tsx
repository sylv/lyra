import type { FC, ReactNode } from "react";

type IconTextProps = {
	icon: ReactNode;
	text: string;
};

export const IconText: FC<IconTextProps> = ({ icon, text }) => {
	return (
		<div className="flex flex-col items-center justify-center h-full min-h-full select-none">
			{icon}
			<div className="m-2 text-zinc-300 lowercase text-xs font-semibold max-w-[240px] text-center">{text}</div>
		</div>
	);
};
