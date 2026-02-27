import { CalendarClockIcon, CalendarPlusIcon, SortAscIcon, StarIcon } from "lucide-react";
import { Fragment, type FC } from "react";
import { OrderBy, type ItemNodeFilter, type RootNodeFilter } from "../@generated/gql/graphql";
import { FilterButton, FilterSelect } from "./filter-button";

interface MediaFilterListProps {
	value: RootNodeFilter | Partial<ItemNodeFilter>;
	onChange: (value: RootNodeFilter | Partial<ItemNodeFilter>) => void;
}

export const MediaFilterList: FC<MediaFilterListProps> = ({ value, onChange }) => {
	const produceChange = (partial: Partial<RootNodeFilter>) => {
		onChange({ ...value, ...partial });
	};

	return (
		<Fragment>
			<FilterButton
				active={value.watched === true}
				onClick={() => {
					if (value.watched != null) produceChange({ watched: null });
					else produceChange({ watched: true });
				}}
			>
				Watched
			</FilterButton>
			<FilterButton
				active={value.watched === false}
				onClick={() => {
					if (value.watched != null) produceChange({ watched: null });
					else produceChange({ watched: false });
				}}
			>
				Unwatched
			</FilterButton>
			<FilterSelect
				label="Order By"
				value={value.orderBy || OrderBy.Alphabetical}
				options={[
					{
						value: OrderBy.Alphabetical,
						label: "Alphabetical",
						icon: SortAscIcon,
					},
					{
						value: OrderBy.Rating,
						label: "Rating",
						icon: StarIcon,
					},
					{
						value: OrderBy.ReleasedAt,
						label: "Release Date",
						icon: CalendarClockIcon,
					},
					{
						value: OrderBy.AddedAt,
						label: "Added Date",
						icon: CalendarPlusIcon,
					},
				]}
				onValueChange={(value) => {
					produceChange({ orderBy: value });
				}}
			/>
		</Fragment>
	);
};
