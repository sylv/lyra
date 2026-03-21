import { XIcon } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import type { CSSProperties, FC, ReactNode } from "react";
import React, { createContext } from "react";
import { createPortal } from "react-dom";
import { Drawer, DrawerContent } from "./ui/drawer";
import { useIsMobile } from "../hooks/use-mobile";
import { useOnClickOutside } from "../hooks/use-on-click-outside";
import { cn } from "../lib/utils";

export interface ModalProps {
	open: boolean;
	children: ReactNode;
	onOpenChange: (open: boolean) => void;
	size?: string;
	rotation?: ModalRotation;
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
	className,
	style,
}) => {
	const ref = React.useRef<HTMLDivElement>(null);
	const isMobile = useIsMobile();
	useOnClickOutside(ref, () => onOpenChange(false));

	const contentStyle: CSSProperties = {
		...style,
		maxHeight: isMobile ? "calc(100dvh - 0.5rem)" : "calc(100vh - 2rem)",
		maxWidth: isMobile ? "100vw" : "calc(100vw - 2rem)",
	};

	if (size) {
		contentStyle.height = size;
		// on mobile the modal becomes a bottom sheet, so aspect-ratio sizing stops making sense.
		if (!isMobile) {
			const aspectRatio = rotation === ModalRotation.Horizontal ? "4 / 2.5" : "2.5 / 4";
			contentStyle.aspectRatio = aspectRatio;
		}
	}

	if (isMobile) {
		contentStyle.width = "100vw";
		return (
			<Drawer open={open} onOpenChange={onOpenChange} direction="bottom" modal={true}>
				<DrawerContent
					className={cn(
						className,
						"max-h-[calc(100dvh-0.5rem)] !w-full max-w-none rounded-none border-0! bg-black text-zinc-100 shadow-2xl shadow-black/50",
					)}
					style={contentStyle}
				>
					<ModalContext.Provider value={{ onOpenChange }}>{children}</ModalContext.Provider>
				</DrawerContent>
			</Drawer>
		);
	}

	return createPortal(
		<AnimatePresence initial={false}>
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
						ref={ref}
						style={contentStyle}
					>
						<ModalContext.Provider value={{ onOpenChange }}>{children}</ModalContext.Provider>
					</motion.div>
				</motion.div>
			)}
		</AnimatePresence>,
		document.body,
	);
};

interface ModalHeaderProps {
	children: ReactNode;
	className?: string;
	contentClassName?: string;
	closeLabel?: string;
	height?: CSSProperties["height"];
	closeButton?: boolean;
}

export const ModalHeader: FC<ModalHeaderProps> = ({
	children,
	className,
	contentClassName,
	closeLabel = "Close",
	height,
	closeButton = true,
}) => {
	const context = React.useContext(ModalContext);
	const isMobile = useIsMobile();
	if (!context) {
		throw new Error("ModalHeader must be used within a Modal");
	}

	return (
		<div
			className={cn("flex h-14 w-full shrink-0 items-center justify-between", className)}
			style={height ? { height } : undefined}
		>
			<div className={cn("flex min-w-0 grow items-center px-6 font-semibold", contentClassName)}>{children}</div>
			{closeButton && !isMobile && (
				<button
					type="button"
					className="flex self-stretch items-center justify-center px-6 text-xs font-semibold text-zinc-400 transition hover:text-zinc-300 hover:underline"
					onClick={() => context.onOpenChange(false)}
				>
					<XIcon className="mr-2 size-4" /> {closeLabel}
				</button>
			)}
		</div>
	);
};

interface ModalBodyProps {
	children: ReactNode;
	className?: string;
}

export const ModalBody: FC<ModalBodyProps> = ({ children, className }) => {
	return <div className={cn("min-h-0 grow overflow-auto px-6 pt-2 pb-6", className)}>{children}</div>;
};

export const ModalFooter: FC<{ children: ReactNode; className?: string }> = ({ children, className }) => {
	return <div className={cn("flex shrink-0 items-center justify-end gap-2 px-6 pb-4", className)}>{children}</div>;
};
