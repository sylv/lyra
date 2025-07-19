import { AudioLines, Home, type LucideIcon } from "lucide-react";
import type { FC, ReactNode } from "react";

const SidebarLink: FC<{
	href: string;
	icon: LucideIcon;
	children: ReactNode;
}> = ({ href, icon: Icon, children }) => (
	<a
		href={href}
		className="text-zinc-300 font-semibold text-sm flex gap-2 items-center hover:text-zinc-100 bg-zinc-700/30 hover:bg-zinc-700/50 py-3 px-6 -mx-6 transition-colors"
	>
		<Icon className="w-5 h-5" />
		{children}
	</a>
);

export const Sidebar: FC<{ children: ReactNode }> = ({ children }) => {
	return (
		<div className="flex w-dvw h-dvh overflow-hidden">
			<div className="w-72 z-10 border-r bg-zinc-800/30 border-700/30 p-6">
				<div className="flex items-center gap-3">
					<AudioLines />
					<div className="flex flex-col">
						<div className="text-zinc-300 font-semibold text-lg -mt-2">
							Lyra
						</div>
						<div className="text-zinc-400 font-semibold text-xs leading-1">
							preview
						</div>
					</div>
				</div>
				<div className="flex flex-col gap-2 mt-8">
					<SidebarLink href="/" icon={Home}>
						Home
					</SidebarLink>
				</div>
			</div>
			<main className="overflow-auto w-full z-10">{children}</main>
		</div>
	);
};
