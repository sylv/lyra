import { useQuery, useSuspenseQuery } from "@apollo/client/react";
import { Link, useLocation } from "@tanstack/react-router";
import { Activity, AudioLines, HomeIcon, SearchIcon, SettingsIcon, type LucideIcon } from "lucide-react";
import { useEffect, useState, type FC, type ReactNode } from "react";
import { generateGradientIcon } from "../lib/generate-gradient-icon";
import { cn } from "../lib/utils";
import { ActivityPanel, ActivityPanelQuery } from "./activity-panel";
import { SuspenseBoundary } from "./fallback";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "./ui/dropdown-menu";
import { graphql } from "../@generated/gql";
import { Spinner } from "./ui/spinner";
import BrandLogo from '../assets/logo.svg'

const SidebarLink: FC<{
	to: string;
	icon?: LucideIcon;
	image?: string;
	active?: boolean;
	children: ReactNode;
	subtext?: ReactNode;
}> = ({ to, icon: Icon, image, active = false, children, subtext }) => {
	if (!Icon && !image) {
		throw new Error("SidebarLink requires either an icon or an image");
	}

	return (
		<Link to={to as never} className="flex items-center relative gap-3 group">
			<div
				className={cn(
					"bg-zinc-600/50 size-9 rounded-md border border-transparent relative overflow-hidden flex items-center justify-center",
					!active && "opacity-80",
				)}
			>
				{Icon && <Icon className="size-4 text-zinc-400" />}
				{image && (
					<img
						src={image}
						className={cn("h-full w-full transition-all duration-500", active && "rotate-90 scale-175")}
						alt=""
					/>
				)}
			</div>
			<div>
				<div className="text-sm group-hover:underline">{children}</div>
				{subtext && <div className="text-xs text-zinc-400 font-semibold">{subtext}</div>}
			</div>
		</Link>
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
	const pathname = useLocation({
		select: (location) => location.pathname,
	});
	const [isActivityOpen, setIsActivityOpen] = useState(false);
	const isSettingsPage = pathname.startsWith("/settings");
	const { data } = useSuspenseQuery(LibrariesQuery);
	const [pollInterval, setPollInterval] = useState(10000);
	const [activitiesRunning, setActivitiesRunning] = useState(false);
	const { data: activityData } = useQuery(ActivityPanelQuery, {
		pollInterval: pollInterval,
	});

	useEffect(() => {
		if (!activityData) return
		const hasRunning = activityData.activities.some(a => a.current < a.total)
		setActivitiesRunning(hasRunning)
		setPollInterval(hasRunning ? 1000 : 10000)
	}, [activityData])

	return (
		<div className="flex w-dvw h-dvh overflow-hidden">
			<div className="w-96 z-10 p-6">
				<div className="flex items-center justify-between gap-1">
					<div className="flex items-center gap-3">
						<img src={BrandLogo} />
						<div className="flex flex-col">
							<div className="text-zinc-100 font-semibold text-lg -mt-2">Lyra</div>
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
										"flex items-center p-2 rounded transition hover:bg-zinc-200/10 relative",
										isActivityOpen && "bg-zinc-200/10",
									)}
								>
									{/* todo: kinda gross but we can't use the bg trick to add an outline to a little spinner because the background can change */}
									<Activity className="size-4" />
									{activitiesRunning && (
										<div className="absolute top-1 right-1 rounded-md">
											<Spinner className="size-2.5" />
										</div>
									)}
								</button>
							</DropdownMenuTrigger>
							<DropdownMenuContent
								align="start"
								side="right"
								sideOffset={12}
								className="border-0 bg-transparent p-0 shadow-none"
							>
								<ActivityPanel open={isActivityOpen} data={activityData} />
							</DropdownMenuContent>
						</DropdownMenu>
						<Link
							href="/settings"
							className={cn(
								"flex items-center p-2 rounded transition hover:bg-zinc-200/10",
								isSettingsPage && "bg-zinc-200/10",
							)}
						>
							<SettingsIcon className="size-4" />
						</Link>
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
					<SidebarLink to="/" icon={HomeIcon} active={pathname === "/"}>
						Home
					</SidebarLink>
					{data?.libraries?.map((library) => {
						const libraryPath = `/library/${library.id}`;
						const isActive = pathname.startsWith(libraryPath);
						const icon = generateGradientIcon(library.createdAt.toString(), { size: 32 });
						return (
							<SidebarLink to={libraryPath} image={icon} active={isActive} key={library.id}>
								{library.name}
							</SidebarLink>
						);
					})}
				</div>
			</div>
			<main className="overflow-auto w-full z-10 pr-6">
				<SuspenseBoundary>{children}</SuspenseBoundary>
			</main>
		</div>
	);
};
