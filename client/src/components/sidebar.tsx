import { useQuery, useSuspenseQuery } from "@apollo/client/react";
import { Link, useLocation } from "@tanstack/react-router";
import { Activity, HomeIcon, MenuIcon, SearchIcon, SettingsIcon, type LucideIcon } from "lucide-react";
import { useState, type FC, type ReactNode } from "react";
import BrandLogo from "../assets/logo.svg";
import { graphql } from "../@generated/gql";
import { LibraryIcon } from "./library-icon";
import { ADMIN_BIT } from "../lib/user-permissions";
import { cn } from "../lib/utils";
import { ActivityPanel, ActivityPanelQuery } from "./activity-panel";
import { SuspenseBoundary } from "./fallback";
import { SearchModal } from "./search-modal";
import { Drawer, DrawerContent } from "./ui/drawer";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "./ui/dropdown-menu";
import { Spinner } from "./ui/spinner";

const SidebarLink: FC<{
	to: string;
	icon?: LucideIcon;
	media?: ReactNode;
	active?: boolean;
	children: ReactNode;
	subtext?: ReactNode;
	onClick?: () => void;
}> = ({ to, icon: Icon, media, active = false, children, subtext, onClick }) => {
	if (!Icon && !media) {
		throw new Error("SidebarLink requires either an icon or media");
	}

	return (
		<Link to={to} className="flex items-center relative gap-3 group" onClick={onClick}>
			<div
				className={cn(
					"bg-zinc-600/50 size-9 rounded-md border border-transparent relative overflow-hidden flex items-center justify-center",
					!active && "opacity-80",
				)}
			>
				{Icon && <Icon className="size-4 text-zinc-400" />}
				{media ? <div className={cn("h-full w-full transition-all duration-500", active && "rotate-90 scale-175")}>{media}</div> : null}
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

const SidebarViewerQuery = graphql(`
	query SidebarViewer {
		viewer {
			id
			permissions
		}
	}
`);

// rendered in the mobile navbar and the desktop sidebar header — owns activity/settings state
const SidebarHeader: FC<{ onNavigate?: () => void }> = ({ onNavigate }) => {
	const pathname = useLocation({ select: (location) => location.pathname });
	const [isActivityOpen, setIsActivityOpen] = useState(false);
	const isSettingsPage = pathname.startsWith("/settings");
	const { data: viewerData } = useSuspenseQuery(SidebarViewerQuery);
	const isAdmin = ((viewerData.viewer?.permissions ?? 0) & ADMIN_BIT) !== 0;
	const { data: activityData } = useQuery(ActivityPanelQuery, { skip: !isAdmin });
	const activitiesRunning = activityData?.activities.some((activity) => activity.current < activity.total) ?? false;

	return (
		<div className="flex items-center justify-between gap-1">
			<div className="items-center gap-3 hidden md:flex">
				<img className="size-9" src={BrandLogo} alt="" />
				<div className="flex flex-col">
					<div className="text-zinc-100 font-semibold text-lg -mt-2">Lyra</div>
					<div className="text-zinc-400 font-semibold text-xs leading-1">preview</div>
				</div>
			</div>
			<div className="flex items-center gap-1">
				{isAdmin ? (
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
				) : null}
				<Link
					to="/settings/about"
					className={cn(
						"flex items-center p-2 rounded transition hover:bg-zinc-200/10",
						isSettingsPage && "bg-zinc-200/10",
					)}
					onClick={onNavigate}
				>
					<SettingsIcon className="size-4" />
				</Link>
			</div>
		</div>
	);
};

const MobileNavbar: FC<{ onMenuClick: () => void }> = ({ onMenuClick }) => {
	return (
		<div className="md:hidden flex items-center justify-between px-3 h-14 shrink-0 z-10">
			<div className="flex items-center gap-2">
				<button
					type="button"
					className="flex items-center p-2 rounded transition hover:bg-zinc-200/10"
					onClick={onMenuClick}
				>
					<MenuIcon className="size-5" />
				</button>
				<img className="size-7" src={BrandLogo} alt="" />
				<span className="text-zinc-100 font-semibold">Lyra</span>
			</div>
			<SidebarHeader />
		</div>
	);
};

const SidebarNav: FC<{ onNavigate?: () => void }> = ({ onNavigate }) => {
	const pathname = useLocation({
		select: (location) => location.pathname,
	});
	const [isSearchOpen, setIsSearchOpen] = useState(false);
	const { data } = useSuspenseQuery(LibrariesQuery);

	return (
		<>
			<div className="mt-6 -mx-1.5">
				<button
					type="button"
					className={cn(
						"w-full border border-zinc-700/50 text-zinc-400 rounded-full px-4 py-2 flex items-center justify-between text-xs hover:bg-zinc-400/10 transition-colors cursor-text",
						isSearchOpen && "bg-zinc-400/10",
					)}
					onClick={() => setIsSearchOpen(true)}
				>
					<span>Search</span>
					<SearchIcon className="size-3" />
				</button>
			</div>
			<div className="flex flex-col gap-2 mt-8 -mx-1.5">
				<div className="font-semibold text-xs text-zinc-300">Media</div>
				<SidebarLink to="/" icon={HomeIcon} active={pathname === "/"} onClick={onNavigate}>
					Home
				</SidebarLink>
				{data?.libraries?.map((library) => {
					const libraryPath = `/library/${library.id}`;
					const isActive = pathname.startsWith(libraryPath);
					return (
						<SidebarLink
							to={libraryPath}
							media={<LibraryIcon createdAt={library.createdAt} className="h-full w-full" size={32} />}
							active={isActive}
							key={library.id}
							onClick={onNavigate}
						>
							{library.name}
						</SidebarLink>
					);
				})}
			</div>
			<SearchModal open={isSearchOpen} onOpenChange={setIsSearchOpen} />
		</>
	);
};

export const Sidebar: FC<{ children: ReactNode }> = ({ children }) => {
	const [isMobileOpen, setIsMobileOpen] = useState(false);

	return (
		<div className="flex flex-col md:flex-row w-dvw h-dvh overflow-hidden">
			{/* mobile top navbar */}
			<MobileNavbar onMenuClick={() => setIsMobileOpen(true)} />

			{/* desktop sidebar */}
			<div className="hidden md:flex md:flex-col w-72 shrink-0 z-10 p-6">
				<SidebarHeader />
				<SidebarNav />
			</div>

			{/* mobile sidebar drawer */}
			<Drawer direction="left" open={isMobileOpen} onOpenChange={setIsMobileOpen}>
				<DrawerContent className="p-6 bg-zinc-950/60 backdrop-blur-2xl">
					<SidebarNav onNavigate={() => setIsMobileOpen(false)} />
				</DrawerContent>
			</Drawer>

			<main className="flex-1 overflow-auto z-10 px-6 md:pl-0">
				<SuspenseBoundary>{children}</SuspenseBoundary>
			</main>
		</div>
	);
};
