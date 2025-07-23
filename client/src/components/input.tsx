import { type FC, type InputHTMLAttributes } from "react";
import { cn } from "../lib/utils";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {}

export const Input: FC<InputProps> = ({ className, ...rest }) => (
	<input
		className={cn(
			"relative h-10 text-sm px-4 rounded-sm w-72",
			"flex items-center justify-center placeholder:text-accent-foreground/40",
			"border border-accent-foreground/20 hover:border-accent-foreground/30 focus:border-accent-foreground/30 outline-none",
			className,
		)}
		{...rest}
	/>
);
