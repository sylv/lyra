import type { FC, ReactNode } from "react";
import { ModalHeader } from "../modal";

interface SetupPageProps {
	title: string;
	description: string;
	children: ReactNode;
}

export const SetupPage: FC<SetupPageProps> = ({ title, description, children }) => (
	<div className="flex min-h-0 grow flex-col">
		<ModalHeader closeButton={false} height="6.5rem" contentClassName="flex-col items-start justify-center gap-1 px-6">
			<h1 className="text-2xl font-bold">{title}</h1>
			<p className="text-sm font-normal text-zinc-400">{description}</p>
		</ModalHeader>
		{children}
	</div>
);
