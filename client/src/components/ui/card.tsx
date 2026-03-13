import type { ComponentProps, FC } from "react";

import { cn } from "@/lib/utils";

export const Card: FC<ComponentProps<"div">> = ({ className, ...props }) => {
	return (
		<div
			data-slot="card"
			className={cn("bg-card text-card-foreground flex flex-col gap-6 rounded border py-6 shadow-sm", className)}
			{...props}
		/>
	);
};

export const CardHeader: FC<ComponentProps<"div">> = ({ className, ...props }) => {
	return (
		<div
			data-slot="card-header"
			className={cn(
				"@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-1.5 px-6 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6",
				className,
			)}
			{...props}
		/>
	);
};

export const CardTitle: FC<ComponentProps<"div">> = ({ className, ...props }) => {
	return <div data-slot="card-title" className={cn("leading-none font-semibold", className)} {...props} />;
};

export const CardDescription: FC<ComponentProps<"div">> = ({ className, ...props }) => {
	return <div data-slot="card-description" className={cn("text-muted-foreground text-sm", className)} {...props} />;
};

export const CardAction: FC<ComponentProps<"div">> = ({ className, ...props }) => {
	return (
		<div
			data-slot="card-action"
			className={cn("col-start-2 row-span-2 row-start-1 self-start justify-self-end", className)}
			{...props}
		/>
	);
};

export const CardContent: FC<ComponentProps<"div">> = ({ className, ...props }) => {
	return <div data-slot="card-content" className={cn("px-6", className)} {...props} />;
};

export const CardFooter: FC<ComponentProps<"div">> = ({ className, ...props }) => {
	return (
		<div data-slot="card-footer" className={cn("flex items-center px-6 [.border-t]:pt-6", className)} {...props} />
	);
};
