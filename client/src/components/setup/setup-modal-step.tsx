import { ArrowRight } from "lucide-react";
import { Button } from "../button";
import type { FC, ReactNode } from "react";

interface SetupModalStepProps {
	children: ReactNode;
	footer?: ReactNode;
	loading: boolean;
	disabled: boolean;
	error: string | null;
	onSubmit: () => void;
}

export const SetupModalStep: FC<SetupModalStepProps> = ({ children, footer, loading, disabled, error, onSubmit }) => (
	<>
		<div className="flex flex-col justify-center items-center bg-zinc-950/40 p-6 rounded-md flex-grow">{children}</div>
		<div className="flex justify-between gap-2 items-center mt-6">
			<div className="h-full flex-grow flex items-center">
				{error && <p className="text-red-400 font-mono bg-red-900/20 px-3 py-1 rounded">{error}</p>}
			</div>
			<div className="flex gap-3 items-center">
				{footer}
				<Button icon={["arrow-right", ArrowRight]} loading={loading} disabled={disabled} onClick={onSubmit}>
					Continue
				</Button>
			</div>
		</div>
	</>
);
