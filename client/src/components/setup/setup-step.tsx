import { ArrowRight } from "lucide-react";
import type { FC, ReactNode } from "react";
import { Button } from "../button";
import { cn } from "@/lib/utils";
import { ModalBody, ModalFooter } from "../modal";

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
		<ModalBody className={cn(centered && "flex items-center justify-center")}>{children}</ModalBody>
		<ModalFooter className="justify-between gap-4 pt-2">
			<div className="flex h-full flex-grow items-center">
				{error && <p className="rounded bg-red-900/20 px-3 py-1 font-mono text-red-400">{error}</p>}
			</div>
			<div className="flex items-center gap-3">
				{footer}
				<Button icon={["arrow-right", ArrowRight]} loading={loading} disabled={disabled} onClick={onSubmit}>
					Continue
				</Button>
			</div>
		</ModalFooter>
	</>
);
