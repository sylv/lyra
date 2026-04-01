import { CalendarClockIcon, CalendarPlusIcon, ListOrderedIcon, SortAscIcon, StarIcon } from "lucide-react";
import { useCallback, useMemo, useState, type FC } from "react";
import { NodeKind, OrderBy, OrderDirection, type NodeFilter } from "../../@generated/gql/graphql";
import { FilterButton, FilterSelect } from "../filter-button";
import { NodePage } from "./node-page";
import z from "zod";
import { useQueryState } from "../../hooks/use-query-state";

const POSTER_WIDTH = 185;
const GAP_SIZE = 16;
const PER_PAGE = 30;

type FilterVariants =
	| {
			type: "seasons";
			totalSeasons: number;
	  }
	| { type: "movies_posters" }
	| { type: "episodes" };

type NodeListProps = FilterVariants & {
	perPage?: number;
	filterOverride?: NodeFilter;
};

export interface PageVariables {
	after: string | null;
}

export enum DisplayKind {
	Poster,
	Episode,
}

const FILTER_NODE_MAP: Record<FilterVariants["type"], NodeKind[]> = {
	movies_posters: [NodeKind.Movie, NodeKind.Series],
	seasons: [NodeKind.Season],
	episodes: [NodeKind.Episode],
};

const FILTER_DISPLAY_MAP: Record<FilterVariants["type"], DisplayKind> = {
	movies_posters: DisplayKind.Poster,
	seasons: DisplayKind.Poster,
	episodes: DisplayKind.Episode,
};

const KIND_NAME_MAP: Record<NodeKind, string> = {
	[NodeKind.Movie]: "Movies",
	[NodeKind.Series]: "Series",
	[NodeKind.Season]: "Seasons",
	[NodeKind.Episode]: "Episodes",
};

export const NodeList: FC<NodeListProps> = ({ perPage, filterOverride, ...variant }) => {
	const displayKind = FILTER_DISPLAY_MAP[variant.type];
	const possibleKinds = FILTER_NODE_MAP[variant.type];
	const schema = useMemo(() => {
		const FilterSchema = z.object({
			kinds: z.array(z.enum(NodeKind)).default(possibleKinds),
			watched: z.boolean().nullable(),
			orderBy: z.enum(OrderBy).default(OrderBy.ReleasedAt),
			orderDirection: z.enum(OrderDirection).nullable(),
		});

		return FilterSchema;
	}, [variant.type]);
	const [filter, setFilter] = useQueryState({ schema, overrides: filterOverride });
	const [selectedKinds, setSelectedKinds] = useState<NodeKind[]>([]);

	const [pageVariables, setPageVariables] = useState<PageVariables[]>([
		{
			after: null,
		},
	]);

	const updateFilter = useCallback((change: Omit<Partial<NodeFilter>, "kinds"> & { kinds?: NodeKind[] }) => {
		setFilter((prev) => ({ ...prev, ...change }));
		setPageVariables([
			{
				after: null,
			},
		]);
	}, []);

	const toggleKind = useCallback(
		(kind: NodeKind) => {
			let nextSelectedKinds: NodeKind[];
			if (selectedKinds.includes(kind)) nextSelectedKinds = selectedKinds.filter((k) => k !== kind);
			else nextSelectedKinds = [...selectedKinds, kind];

			setSelectedKinds(nextSelectedKinds);
			updateFilter({ kinds: nextSelectedKinds.length > 0 ? nextSelectedKinds : possibleKinds });
		},
		[selectedKinds, possibleKinds],
	);

	return (
		<>
			<div className="my-4 flex flex-col gap-2">
				<div className="flex flex-wrap gap-2">
					{(!filterOverride || filterOverride.watched == null) && (
						<>
							<FilterButton
								active={filter.watched === true}
								onClick={() => updateFilter({ watched: filter.watched != null ? null : true })}
							>
								Watched
							</FilterButton>
							<FilterButton
								active={filter.watched === false}
								onClick={() => updateFilter({ watched: filter.watched != null ? null : false })}
							>
								Unwatched
							</FilterButton>
						</>
					)}
					{possibleKinds.length > 1 && (
						<>
							{possibleKinds.map((kind) => (
								<FilterButton key={kind} active={selectedKinds.includes(kind)} onClick={() => toggleKind(kind)}>
									{KIND_NAME_MAP[kind]}
								</FilterButton>
							))}
						</>
					)}
					{(!filterOverride || filterOverride.orderBy == null) && (
						<FilterSelect
							label="Order By"
							value={filter.orderBy || OrderBy.Alphabetical}
							options={[
								{ value: OrderBy.Alphabetical, label: "Alphabetical", icon: SortAscIcon },
								{ value: OrderBy.Rating, label: "Rating", icon: StarIcon },
								{ value: OrderBy.ReleasedAt, label: "Release Date", icon: CalendarClockIcon },
								{ value: OrderBy.AddedAt, label: "Added Date", icon: CalendarPlusIcon },
								{ value: OrderBy.Order, label: "Canonical Order", icon: ListOrderedIcon },
							]}
							onValueChange={(nextValue) => updateFilter({ orderBy: nextValue })}
						/>
					)}
				</div>
			</div>
			<div className="flex flex-wrap gap-4">
				<div className="w-full relative mb-24">
					<div
						className={displayKind === DisplayKind.Poster ? "grid" : "space-y-6"}
						style={
							displayKind === DisplayKind.Poster
								? {
										gridTemplateColumns: `repeat(auto-fill, minmax(${POSTER_WIDTH}px, 1fr))`,
										columnGap: GAP_SIZE,
										rowGap: GAP_SIZE,
									}
								: undefined
						}
					>
						{pageVariables.map((variables, index) => (
							<NodePage
								key={index}
								displayKind={displayKind}
								filter={filter}
								variables={variables}
								isLast={index === pageVariables.length - 1}
								perPage={perPage || PER_PAGE}
								onLoadMore={(after) => {
									setPageVariables((prev) => {
										return [...prev, { after }];
									});
								}}
							/>
						))}
					</div>
				</div>
			</div>
		</>
	);
};
