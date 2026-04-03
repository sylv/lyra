import { CalendarClockIcon, CalendarPlusIcon, ListOrderedIcon, SortAscIcon, StarIcon } from "lucide-react";
import { useMemo, useState, type FC } from "react";
import z from "zod";
import {
	NodeAvailability,
	NodeKind,
	OrderBy,
	OrderDirection,
	type NodeFilter,
} from "../../@generated/gql/graphql";
import { useQueryState } from "../../hooks/use-query-state";
import { FilterButton, FilterSelect } from "../filter-button";
import { NodePage } from "./node-page";

const POSTER_WIDTH = 185;
const GAP_SIZE = 16;
const PER_PAGE = 30;

type FilterVariants =
	| {
			type: "seasons";
			totalSeasons: number;
	  }
	| { type: "movies_series" }
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
	movies_series: [NodeKind.Movie, NodeKind.Series],
	seasons: [NodeKind.Season],
	episodes: [NodeKind.Episode],
};

const FILTER_DISPLAY_MAP: Record<FilterVariants["type"], DisplayKind> = {
	movies_series: DisplayKind.Poster,
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
			availability: z.enum(NodeAvailability).default(NodeAvailability.Available),
			watched: z.boolean().nullable(),
			orderBy: z.enum(OrderBy).default(OrderBy.LastAired),
			orderDirection: z.enum(OrderDirection).nullable(),
		});

		return FilterSchema;
	}, [variant.type]);
	const [queryFilter, setFilter] = useQueryState({ schema });
	const [selectedKinds, setSelectedKinds] = useState<NodeKind[]>([]);
	const [pageVariables, setPageVariables] = useState<PageVariables[]>([
		{
			after: null,
		},
	]);

	const filter: NodeFilter = { ...queryFilter, ...filterOverride };
	const updateFilter = (change: Omit<Partial<NodeFilter>, "kinds"> & { kinds?: NodeKind[] }) => {
		setFilter((prev) => ({ ...prev, ...change }));
		setPageVariables([
			{
				after: null,
			},
		]);
	};

	const toggleKind = (kind: NodeKind) => {
		let nextSelectedKinds: NodeKind[];
		if (selectedKinds.includes(kind)) nextSelectedKinds = selectedKinds.filter((k) => k !== kind);
		else nextSelectedKinds = [...selectedKinds, kind];

		setSelectedKinds(nextSelectedKinds);
		updateFilter({ kinds: nextSelectedKinds.length > 0 ? nextSelectedKinds : possibleKinds });
	};

	return (
		<>
			<div className="my-4 flex flex-col gap-2">
				<div className="flex flex-wrap gap-2">
					{(!filterOverride || filterOverride.watched == null) && (
						<>
							<FilterButton
								active={filter.watched === true}
								onClick={() => updateFilter({ watched: filter.watched === true ? null : true })}
							>
								Watched
							</FilterButton>
							<FilterButton
								active={filter.watched === false}
								onClick={() => updateFilter({ watched: filter.watched === false ? null : false })}
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
							value={queryFilter.orderBy || OrderBy.LastAired}
							options={[
								{ value: OrderBy.Alphabetical, label: "Alphabetical", icon: SortAscIcon },
								{ value: OrderBy.Rating, label: "Rating", icon: StarIcon },
								{ value: OrderBy.FirstAired, label: "First Aired", icon: CalendarClockIcon },
								{ value: OrderBy.LastAired, label: "Last Aired", icon: CalendarClockIcon },
								{ value: OrderBy.AddedAt, label: "Added Date", icon: CalendarPlusIcon },
								{ value: OrderBy.Order, label: "Canonical Order", icon: ListOrderedIcon },
							]}
							onValueChange={(nextValue) => updateFilter({ orderBy: nextValue })}
						/>
					)}
					{(!filterOverride || filterOverride.availability == null) && (
						<FilterSelect
							label="Availability"
							value={queryFilter.availability || NodeAvailability.Available}
							options={[
								{ value: NodeAvailability.Available, label: "Available" },
								{ value: NodeAvailability.Unavailable, label: "Unavailable" },
								{ value: NodeAvailability.Both, label: "Both" },
							]}
							onValueChange={(nextValue) => updateFilter({ availability: nextValue })}
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
								isFirst={index === 0}
								isLast={index === pageVariables.length - 1}
								perPage={perPage || PER_PAGE}
								onLoadMore={(after) => {
									console.log({ after });
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
