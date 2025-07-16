import type { Media } from "../@generated/server";

export const getPathForMedia = (media: Media) => {
	switch (media.media_type) {
		case "Show":
			return `/series/${media.id}`;
		case "Movie":
			return `/movie/${media.id}`;
		case "Season":
			if (media.season_number == null) {
				return `/series/${media.parent_id}`;
			}

			return `/series/${media.parent_id}/season/${media.season_number}`;
		case "Episode":
			if (media.season_number == null || media.episode_number == null) {
				return `/series/${media.parent_id}`;
			}

			return `/series/${media.parent_id}/season/${media.season_number}/episode/${media.episode_number}`;
	}
};
