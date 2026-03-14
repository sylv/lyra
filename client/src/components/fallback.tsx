import { type FC, type ReactNode, Suspense } from "react";
import { cn } from "../lib/utils";
import { IconText } from "./icon-text";
import { LoadingText } from "./loading-text";
import { Spinner } from "./ui/spinner";

export const Fallback: FC<{ className?: string }> = ({ className }) => {
	return (
		<div className={cn("h-full w-full flex items-center justify-center", className)}>
			<IconText icon={<Spinner className="size-4" />} text={<LoadingText />} />
		</div>
	);
};

export const SuspenseBoundary: FC<{ children: ReactNode; className?: string }> = ({ children, className }) => (
	<Suspense fallback={<Fallback className={className} />}>{children}</Suspense>
);
