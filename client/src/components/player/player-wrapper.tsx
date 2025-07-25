import { graphql } from "gql.tada";
import type { FC } from "react";
import { useStore } from "zustand";
import { Player } from "./player";
import { playerState } from "./player-state";

export const PlayerFrag = graphql(`
	fragment Player on Media {
		id
		name
		seasonNumber
		episodeNumber
		parent {
			name
		}
		defaultConnection {
			id
		}
	}
`);

export const PlayerWrapper: FC = () => {
	const { currentMedia: currentMediaRef } = useStore(playerState);
	if (!currentMediaRef) {
		return null;
	}

	return <Player media={currentMediaRef} />;
};
