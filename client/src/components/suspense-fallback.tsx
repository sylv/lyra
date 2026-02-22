import { Suspense, type FC, type ReactNode } from "react";
import { Spinner } from "./ui/spinner";

export const SuspenseFallback: FC = () => (
	<div className="h-full w-full flex items-center justify-center">
		<Spinner className="size-5" />
	</div>
);

export const SuspenseBoundary: FC<{ children: ReactNode }> = ({ children }) => (
	<Suspense fallback={<SuspenseFallback />}>{children}</Suspense>
);
