import { useQuery } from "@apollo/client/react";
import { Dialog, DialogContent, DialogOverlay } from "@radix-ui/react-dialog";
import { graphql } from "gql.tada";
import { Loader2 } from "lucide-react";
import { useMemo, useState, type FC } from "react";
import { create } from "zustand";
import { useDebounce } from "../../hooks/use-debounce";
import { formatReleaseYear } from "../../lib/format-release-year";
import { getPathForMedia, GetPathForMediaFrag } from "../../lib/getPathForMedia";
import { cn } from "../../lib/utils";
import { Poster } from "../poster";
import { Thumbnail } from "../thumbnail";

export const useSearchStore = create<boolean>(() => false);

export const setIsSearchOpen = (isOpen: boolean) => {
	useSearchStore.setState(isOpen);
};

const Query = graphql(
	`
    query SearchMedia($term: String!) {
        nodeList(filter: { search: $term }, first: 50) {
            edges {
                node {
                    id
                    kind
                    properties {
						description
						rating
						releasedAt
						endedAt
						thumbnailUrl
						posterUrl
                    }
					name
                    parent {
                        name
                    }
					...GetPathForMedia
                }
            }
        }
    }    
`,
	[GetPathForMediaFrag],
);

export const SearchModal: FC = () => {
	const [search, setSearch] = useState("");
	const isOpen = useSearchStore();
	const [debouncedSearch, isDebouncing] = useDebounce(search, 500);

	const { data, loading } = useQuery(Query, {
		skip: !debouncedSearch,
		variables: {
			term: search,
		},
	});

	const showLoader = isDebouncing || loading;

	const groups = useMemo(() => {
		if (!data) return [];

		const groups = new Map<string, (typeof data.nodeList.edges)[number]["node"][]>();
		for (const edge of data.nodeList.edges) {
			const group = groups.get(edge.node.kind);
			if (group) {
				group.push(edge.node);
			} else {
				groups.set(edge.node.kind, [edge.node]);
			}
		}

		return Array.from(groups.entries())
			.map(([kind, nodes]) => ({
				kind,
				nodes,
			}))
			.sort((a, b) => {
				// force episodes to be last, normally they match the most but are not relevant
				// otherwise we use the original ordering (ie, if a movie is first, the movie group shows first)
				if (a.kind === "EPISODE") return 1;
				if (b.kind === "EPISODE") return -1;
				return 0;
			});
	}, [data]);

	return (
		<Dialog open={isOpen} onOpenChange={setIsSearchOpen}>
			<DialogOverlay className="fixed inset-0 bg-black/50 backdrop-blur-xs z-10" />
			<DialogContent
				className={cn(
					"rear z-20 fixed left-1/2 top-1/2 max-h-[85vh] w-[1080px] h-[650px] max-w-[90vw] -translate-x-1/2 -translate-y-1/2",
					"outline-none rounded-lg bg-black/50 backdrop-blur-2xl overflow-hidden shadow-2xl shadow-black",
				)}
			>
				<div className="relative border-b border-white/10">
					<input
						autoFocus
						type="text"
						value={search}
						onChange={(e) => setSearch(e.target.value)}
						className="px-6 py-6 w-full outline-none focus:bg-white/5 bg-white/5 transition-colors text-sm"
						placeholder="Search for a movie, series or episode"
					/>
					{showLoader && (
						<div className="absolute right-10 top-0 bottom-0 flex items-center justify-center">
							<Loader2 className="w-4 h-4 animate-spin" />
						</div>
					)}
				</div>

				<div className="h-full p-6 space-y-4 w-full overflow-y-auto pb-24">
					{groups.map((group) => (
						<div key={group.kind}>
							<h2 className="mb-1 text-xs font-semibold text-zinc-500">
								{group.kind === "SERIES" ? "SERIES" : `${group.kind}S`}
							</h2>
							<div className="grid grid-cols-2 gap-3">
								{group.nodes.map((node) => {
									const subheader = node.parent
										? node.parent.name
										: formatReleaseYear(node.properties.releasedAt, node.properties.endedAt);
									const path = getPathForMedia(node);

									return (
										<a
											href={path}
											key={node.id}
											className={cn(
												"w-full rounded-lg cursor-pointer transition-all duration-200 text-left",
												"bg-white/5 hover:bg-white/10 group",
											)}
											onClick={() => {
												setIsSearchOpen(false);
												setSearch("");
											}}
										>
											<div className="flex items-center gap-5">
												{node.kind === "EPISODE" ? (
													<Thumbnail imageUrl={node.properties.thumbnailUrl} alt={node.name} className="h-26 rounded-r-none" />
												) : (
													<Poster imageUrl={node.properties.posterUrl} alt={node.name} className="h-26 rounded-r-none" />
												)}
												<div className="flex-1 min-w-0">
													<h5 className="text-xs text-zinc-400">{subheader}</h5>
													<h3 className="font-semibold text-white text-sm group-hover:text-white/90 transition-colors truncate">
														{node.name}
													</h3>
												</div>
											</div>
										</a>
									);
								})}
							</div>
						</div>
					))}
				</div>
			</DialogContent>
		</Dialog>
	);
};
