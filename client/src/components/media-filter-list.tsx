import { Fragment, type FC } from "react";
import type { NodeFilter } from "../@generated/enums";
import { FilterButton, FilterSelect } from "./filter-button";
import { CalendarClockIcon, CalendarPlusIcon, ListVideoIcon, SortAscIcon, StarIcon } from "lucide-react";

interface MediaFilterListProps {
	value: Partial<NodeFilter>;
	onChange: (value: Partial<NodeFilter>) => void;
}

export const MediaFilterList: FC<MediaFilterListProps> = ({ value, onChange }) => {
	const produceChange = (partial: Partial<NodeFilter>) => {
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
				value={value.orderBy || "ALPHABETICAL"}
				options={[
					{
						value: "ALPHABETICAL",
						label: "Alphabetical",
						icon: SortAscIcon,
					},
					{
						value: "RATING",
						label: "Rating",
						icon: StarIcon,
					},
					{
						value: "RELEASED_AT",
						label: "Release Date",
						icon: CalendarClockIcon,
					},
					{
						value: "ADDED_AT",
						label: "Added Date",
						icon: CalendarPlusIcon,
					},
					{
						value: "SEASON_EPISODE",
						label: "Episode Number",
						icon: ListVideoIcon,
					},
				]}
				onValueChange={(value) => {
					produceChange({ orderBy: value });
				}}
			/>
		</Fragment>
	);
};
