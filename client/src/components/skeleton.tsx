import { cn } from "@/lib/utils";

export const SKELETON_CLASSES = "bg-zinc-700/30 rounded-md";

function Skeleton({ className, ...props }: React.ComponentProps<"div">) {
	return <div data-slot="skeleton" className={cn(SKELETON_CLASSES, className)} {...props} />;
}

export { Skeleton };
