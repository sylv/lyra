import { XIcon } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import type { CSSProperties, FC, ReactNode } from "react";
import React, { createContext, useEffect } from "react";
import { createPortal } from "react-dom";
import { cn } from "../lib/utils";

export interface ModalProps {
	open: boolean;
	children: ReactNode;
	onOpenChange: (open: boolean) => void;
	size?: string;
	rotation?: ModalRotation;
	container?: Element | null;
	className?: string;
	style?: CSSProperties;
}

export enum ModalRotation {
	Vertical,
	Horizontal,
}

const ModalContext = createContext<{ onOpenChange: (open: boolean) => void } | null>(null);

export const Modal: FC<ModalProps> = ({
	open,
	children,
	onOpenChange,
	size,
	rotation = ModalRotation.Horizontal,
	container,
	className,
	style,
}) => {
	const portalTarget = container ?? (typeof document !== "undefined" ? document.body : null);

	useEffect(() => {
		if (!open || typeof document === "undefined") {
			return;
		}

		const onKeyDown = (event: KeyboardEvent) => {
			if (event.key === "Escape") {
				onOpenChange(false);
			}
		};

		document.addEventListener("keydown", onKeyDown);
		return () => {
			document.removeEventListener("keydown", onKeyDown);
		};
	}, [open, onOpenChange]);

	if (portalTarget == null) {
		return null;
	}

	const aspectRatio = rotation === ModalRotation.Horizontal ? "4 / 2.5" : "2.5 / 4";
	const contentStyle: CSSProperties = {
		maxHeight: "calc(100vh - 2rem)",
		maxWidth: "calc(100vw - 2rem)",
		...(size
			? {
				height: size,
				aspectRatio,
			}
			: {}),
		...style,
	};

	return createPortal(
		<AnimatePresence>
			{open && (
				<motion.div
					className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-xs"
					onClick={() => onOpenChange(false)}
					initial={{ opacity: 0 }}
					animate={{ opacity: 1 }}
					exit={{ opacity: 0 }}
				>
					<motion.div
						className={cn(
							"flex flex-col overflow-hidden rounded-md bg-black text-zinc-100 shadow-2xl shadow-black/50",
							className,
						)}
						onClick={(event) => event.stopPropagation()}
						initial={{ opacity: 0, scale: 0.75 }}
						animate={{ opacity: 1, scale: 1 }}
						exit={{ opacity: 0, scale: 0.95 }}
						style={contentStyle}
					>
						<ModalContext.Provider value={{ onOpenChange }}>
							{children}
						</ModalContext.Provider>
					</motion.div>
				</motion.div>
			)
			}
		</AnimatePresence >,
		portalTarget,
	);
};

interface ModalHeaderProps {
	children: ReactNode;
	className?: string;
	contentClassName?: string;
	closeLabel?: string;
}

export const ModalHeader: FC<ModalHeaderProps> = ({
	children,
	className,
	contentClassName,
	closeLabel = "Close",
}) => {
	const context = React.useContext(ModalContext);
	if (!context) {
		throw new Error("ModalHeader must be used within a Modal");
	}

	return (
		<div className={cn("flex h-14 w-full items-center justify-between border-b border-zinc-900", className)}>
			<div className={cn("min-w-0 grow px-6 font-semibold", contentClassName)}>{children}</div>
			<button
				type="button"
				className="flex h-14 items-center justify-center px-6 text-xs font-semibold text-zinc-400 transition hover:text-zinc-300 hover:underline"
				onClick={() => context.onOpenChange(false)}
			>
				<XIcon className="mr-2 size-4" /> {closeLabel}
			</button>
		</div>
	);
};

interface ModalBodyProps {
	children: ReactNode;
	className?: string;
	patterned?: boolean;
}

export const ModalBody: FC<ModalBodyProps> = ({ children, className, patterned = true }) => {
	const backgroundColour = "rgba(255, 255, 255, 0.02)";
	return (
		<div
			className={cn("grow px-6 py-4", className)}
			style={
				patterned
					? {
						backgroundImage: `linear-gradient(${backgroundColour} .05em, transparent .05em), linear-gradient(90deg, ${backgroundColour} .05em, transparent .05em)`,
						backgroundSize: "2em 2em",
					}
					: undefined
			}
		>
			{children}
		</div>
	);
};

export const ModalFooter: FC<{ children: ReactNode; className?: string }> = ({ children, className }) => {
	return <div className={cn("flex items-center justify-end gap-2 px-6 pb-4", className)}>{children}</div>;
};
