import { Loader2, type LucideIcon } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import type { FC, ReactNode } from "react";
import { cn } from "../lib/utils";

export enum ButtonStyle {
	Primary = "bg-amethyst-600/50 hover:bg-amethyst-600/70",
	Transparent = "bg-transparent hover:bg-zinc-500/20",
}

interface ButtonProps {
	children: ReactNode;
	icon?: [string, LucideIcon];
	iconSide?: "left" | "right";
	loading?: boolean;
	className?: string;
	style?: ButtonStyle;
	disabled?: boolean;
	onClick?: () => void;
}

export const Button: FC<ButtonProps> = ({
	children,
	className,
	icon,
	iconSide = "right",
	loading,
	style = ButtonStyle.Primary,
	disabled,
	onClick,
}) => {
	if (loading) {
		icon = ["loading-spinner", Loader2];
	}

	const [iconKey, Icon] = icon ?? ["no-icon", null];
	const iconTranslate = iconSide === "right" ? "group-hover:translate-x-0.5" : "group-hover:-translate-x-0.5";
	const iconElement = Icon ? (
		<div
			className={cn(
				"h-4 w-4 flex items-center justify-center relative overflow-hidden",
				!loading && !disabled && `transition-all ${iconTranslate}`,
			)}
		>
			<AnimatePresence>
				<motion.div
					key={iconKey}
					initial={{ y: 20, opacity: 0 }}
					animate={{ y: 0, opacity: 1 }}
					exit={{ y: -20, opacity: 0 }}
					className="flex items-center justify-center absolute inset-0"
					transition={{
						type: "spring",
						stiffness: 200,
						damping: 25,
						duration: 0.175,
					}}
				>
					<Icon className={cn("h-4 w-4", loading && "animate-spin")} />
				</motion.div>
			</AnimatePresence>
		</div>
	) : null;

	return (
		<button
			type="button"
			onClick={onClick}
			className={cn(
				"px-6 py-2 rounded-sm flex items-center gap-3 font-semibold text-sm transition-colors group hover:underline",
				style,
				disabled && "opacity-80 cursor-not-allowed",
				className,
			)}
		>
			{iconSide === "left" && iconElement}
			{children}
			{iconSide === "right" && iconElement}
		</button>
	);
};
