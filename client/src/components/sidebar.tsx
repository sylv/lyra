import { useQuery } from "@apollo/client/react";
import { graphql } from "gql.tada";
import { Activity, AudioLines, HomeIcon, SearchIcon, SettingsIcon, type LucideIcon } from "lucide-react";
import { useState, type FC, type ReactNode } from "react";
import { usePageContext } from "vike-react/usePageContext";
import { generateGradientIcon } from "../lib/generate-gradient-icon";
import { cn } from "../lib/utils";
import { ActivityPanel } from "./activity-panel";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "./ui/dropdown-menu";

const SidebarLink: FC<{
	href: string;
	icon?: LucideIcon;
	image?: string;
	active?: boolean;
	children: ReactNode;
	subtext?: ReactNode;
}> = ({ href, icon: Icon, image, active = false, children, subtext }) => {
	if (!Icon && !image) {
		throw new Error("SidebarLink requires either an icon or an image");
	}

	return (
		<a href={href} className="flex items-center relative gap-3 group">
			<div
				className={cn(
					"bg-zinc-600/50 size-9 rounded-md border border-transparent relative overflow-hidden flex items-center justify-center",
					active && "border-purple-500",
				)}
			>
				{Icon && <Icon className="size-4 text-zinc-400" />}
				{image && <img src={image} className="h-full w-full" alt="" />}
				{active && (
					<div className="absolute right-0 top-0 size-5 translate-x-[50%] -translate-y-[50%] rotate-45 bg-purple-500 z-10" />
				)}
			</div>
			<div>
				<div className="text-sm group-hover:underline">{children}</div>
				{subtext && <div className="text-xs text-zinc-400 font-semibold">{subtext}</div>}
			</div>
		</a>
	);
};

const LibrariesQuery = graphql(`
	query Libraries {
		libraries {
			id
			name
			createdAt
		}
	}	
`);

export const Sidebar: FC<{ children: ReactNode }> = ({ children }) => {
	const pageContext = usePageContext();
	const pathname = pageContext.urlParsed.pathname;
	const [isActivityOpen, setIsActivityOpen] = useState(false);
	const isSettingsPage = pathname.startsWith("/settings");
	const { data } = useQuery(LibrariesQuery);

	return (
		<div className="flex w-dvw h-dvh overflow-hidden">
			<div className="w-96 z-10 p-6">
				<div className="flex items-center justify-between gap-1">
					<div className="flex items-center gap-3">
						<AudioLines />
						<div className="flex flex-col">
							<div className="text-zinc-300 font-semibold text-lg -mt-2">Lyra</div>
							<div className="text-zinc-400 font-semibold text-xs leading-1">preview</div>
						</div>
					</div>
					<div className="flex items-center gap-1">
						<DropdownMenu open={isActivityOpen} onOpenChange={setIsActivityOpen}>
							<DropdownMenuTrigger asChild>
								{/* todo: with activities running, show indicator icon with count */}
								<button
									type="button"
									className={cn(
										"flex items-center p-2 rounded-lg transition hover:bg-zinc-200/10",
										isActivityOpen && "bg-zinc-200/10",
									)}
								>
									<Activity className="w-4 h-4" />
								</button>
							</DropdownMenuTrigger>
							<DropdownMenuContent
								align="start"
								side="right"
								sideOffset={12}
								className="border-0 bg-transparent p-0 shadow-none"
							>
								<ActivityPanel open={isActivityOpen} />
							</DropdownMenuContent>
						</DropdownMenu>
						<a
							href="/settings"
							className={cn(
								"flex items-center p-2 rounded-lg transition hover:bg-zinc-200/10",
								isSettingsPage && "bg-zinc-200/10",
							)}
						>
							<SettingsIcon className="w-4 h-4" />
						</a>
					</div>
				</div>
				<div className="mt-6 -mx-1.5">
					<button
						type="button"
						className="w-full border border-zinc-700/50 text-zinc-400 rounded-full px-4 py-2 flex items-center justify-between text-xs hover:bg-zinc-400/10 transition-colors"
						onClick={() => {}}
					>
						<span>Search</span>
						<SearchIcon className="size-3" />
					</button>
				</div>
				<div className="flex flex-col gap-2 mt-8 -mx-1.5">
					<div className="font-semibold text-xs text-zinc-300">Media</div>
					<SidebarLink href="/" icon={HomeIcon} active={pathname === "/"}>
						Home
					</SidebarLink>
					{data?.libraries?.map((library) => {
						const libraryPath = `/library/${library.id}`;
						const isActive = pathname.startsWith(libraryPath);
						const icon = generateGradientIcon(library.createdAt.toString(), { size: 32 });
						return (
							<SidebarLink href={libraryPath} image={icon} active={isActive} key={library.id}>
								{library.name}
							</SidebarLink>
						);
					})}
				</div>
			</div>
			<main className="overflow-auto w-full z-10">{children}</main>
		</div>
	);
};
