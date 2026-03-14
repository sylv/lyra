import { ArrowRight } from "lucide-react";
import type { FC, ReactNode } from "react";
import { Button } from "../button";
import { cn } from "@/lib/utils";

interface SetupStepProps {
	children: ReactNode;
	footer?: ReactNode;
	loading: boolean;
	disabled: boolean;
	centered?: boolean;
	error?: string | null;
	onSubmit: () => void;
}

export const SetupStep: FC<SetupStepProps> = ({
	children,
	footer,
	loading,
	disabled,
	error,
	centered = true,
	onSubmit,
}) => (
	<>
		<div className={cn("flex flex-col p-6 flex-grow", centered && "items-center justify-center")}>{children}</div>
		<div className="mt-6 flex items-center justify-between gap-2">
			<div className="flex h-full flex-grow items-center">
				{error && <p className="rounded bg-red-900/20 px-3 py-1 font-mono text-red-400">{error}</p>}
			</div>
			<div className="flex items-center gap-3">
				{footer}
				<Button icon={["arrow-right", ArrowRight]} loading={loading} disabled={disabled} onClick={onSubmit}>
					Continue
				</Button>
			</div>
		</div>
	</>
);
