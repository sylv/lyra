import { CalendarClockIcon, CalendarPlusIcon, ListOrderedIcon, SortAscIcon, StarIcon } from "lucide-react";
import { Fragment, type FC } from "react";
import { OrderBy, type NodeFilter } from "../../@generated/gql/graphql";
import { FilterButton, FilterSelect } from "../filter-button";

interface NodeFilterListProps {
	value: NodeFilter;
	onChange: (value: NodeFilter) => void;
}

export const NodeFilterList: FC<NodeFilterListProps> = ({ value, onChange }) => {
	const produceChange = (partial: Partial<NodeFilter>) => onChange({ ...value, ...partial });

	return (
		<Fragment>
			<FilterButton
				active={value.watched === true}
				onClick={() => produceChange({ watched: value.watched != null ? null : true })}
			>
				Watched
			</FilterButton>
			<FilterButton
				active={value.watched === false}
				onClick={() => produceChange({ watched: value.watched != null ? null : false })}
			>
				Unwatched
			</FilterButton>
			<FilterSelect
				label="Order By"
				value={value.orderBy || OrderBy.Alphabetical}
				options={[
					{ value: OrderBy.Alphabetical, label: "Alphabetical", icon: SortAscIcon },
					{ value: OrderBy.Rating, label: "Rating", icon: StarIcon },
					{ value: OrderBy.ReleasedAt, label: "Release Date", icon: CalendarClockIcon },
					{ value: OrderBy.AddedAt, label: "Added Date", icon: CalendarPlusIcon },
					{ value: OrderBy.Order, label: "Canonical Order", icon: ListOrderedIcon },
				]}
				onValueChange={(nextValue) => produceChange({ orderBy: nextValue })}
			/>
		</Fragment>
	);
};
