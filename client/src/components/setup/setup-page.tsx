import type { FC, ReactNode } from "react";

interface SetupPageProps {
	title: string;
	description: string;
	children: ReactNode;
}

export const SetupPage: FC<SetupPageProps> = ({ title, description, children }) => (
	<>
		<h1 className="text-2xl font-bold">{title}</h1>
		<p className="mb-6 text-sm text-zinc-400">{description}</p>
		{children}
	</>
);
