import { CalendarClockIcon, CalendarPlusIcon, ListOrderedIcon, SortAscIcon, StarIcon } from "lucide-react";
import { useMemo, useState, type FC, type ReactNode } from "react";
import z from "zod";
import { NodeAvailability, NodeKind, OrderBy, OrderDirection, type NodeFilter } from "../../@generated/gql/graphql";
import { useQueryState } from "../../hooks/use-query-state";
import { FilterButton, FilterSelect } from "../filter-button";

type FilterVariants = { type: "movies_series" } | { type: "episodes"; totalSeasons: number };

type NodeListFilterProps = FilterVariants & {
	defaultOrderBy: OrderBy;
	filterOverride?: NodeFilter;
	children: (filter: NodeFilter) => ReactNode;
};

const FILTER_NODE_MAP: Record<FilterVariants["type"], NodeKind[]> = {
	movies_series: [NodeKind.Movie, NodeKind.Series],
	episodes: [NodeKind.Episode],
};

const KIND_NAME_MAP: Record<NodeKind, string> = {
	[NodeKind.Movie]: "Movies",
	[NodeKind.Series]: "Series",
	[NodeKind.Season]: "Seasons",
	[NodeKind.Episode]: "Episodes",
};

export const NodeListFilter: FC<NodeListFilterProps> = ({ children, filterOverride, defaultOrderBy, ...variant }) => {
	const possibleKinds = FILTER_NODE_MAP[variant.type];
	const schema = useMemo(() => {
		const FilterSchema = z.object({
			kinds: z.array(z.enum(NodeKind)).default(possibleKinds),
			availability: z.enum(NodeAvailability).default(NodeAvailability.Available),
			watched: z.boolean().nullable(),
			orderBy: z.enum(OrderBy).default(defaultOrderBy),
			orderDirection: z.enum(OrderDirection).nullable(),
		});

		if (variant.type === "episodes") {
			return FilterSchema.extend({
				seasonNumbers: z.array(z.number()).default([1]),
			});
		}

		return FilterSchema;
	}, [variant.type, defaultOrderBy, possibleKinds]);

	const [queryFilter, setFilter] = useQueryState({ schema });
	const [selectedKinds, setSelectedKinds] = useState<NodeKind[]>([]);
	const filter = useMemo(() => ({ ...queryFilter, ...filterOverride }), [queryFilter, filterOverride]);

	const updateFilter = (change: Omit<Partial<NodeFilter>, "kinds"> & { kinds?: NodeKind[] }) => {
		setFilter((prev) => ({ ...prev, ...change }));
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
			<div className="flex flex-wrap gap-2">
				{variant.type === "episodes" &&
					Array.from({ length: variant.totalSeasons }, (_, i) => i + 1).map((seasonNumber) => (
						<FilterButton
							key={seasonNumber}
							active={filter.seasonNumbers?.includes(seasonNumber) ?? false}
							onClick={(event) => {
								if (event.ctrlKey || event.shiftKey || event.metaKey) {
									const seasonNumbers = filter.seasonNumbers ?? [];
									if (seasonNumbers.includes(seasonNumber)) {
										updateFilter({
											seasonNumbers: seasonNumbers.filter((n) => n !== seasonNumber),
										});
									} else {
										updateFilter({ seasonNumbers: [...seasonNumbers, seasonNumber] });
									}
								} else {
									updateFilter({ seasonNumbers: [seasonNumber] });
								}
							}}
						>
							Season {seasonNumber}
						</FilterButton>
					))}
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
				{possibleKinds.length > 1 &&
					possibleKinds.map((kind) => (
						<FilterButton key={kind} active={selectedKinds.includes(kind)} onClick={() => toggleKind(kind)}>
							{KIND_NAME_MAP[kind]}
						</FilterButton>
					))}
				{(!filterOverride || filterOverride.orderBy == null) && (
					<FilterSelect
						label="Order By"
						value={queryFilter.orderBy || defaultOrderBy}
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
			{children(filter)}
		</>
	);
};
