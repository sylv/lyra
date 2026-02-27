import { useEffect } from "react";
import { create } from "zustand";
import { unmask, type FragmentType } from "../@generated/gql";
import type { ImageAssetFragment } from "../@generated/gql/graphql";
import { Fragment } from "../components/image";

export const backgroundStore = create<ImageAssetFragment | null>(() => null);

export const useDynamicBackground = (assetRaw: FragmentType<typeof Fragment> | null, use?: boolean) => {
	const asset = unmask(Fragment, assetRaw);
	useEffect(() => {
		if (use === false) return;
		backgroundStore.setState(asset, true);
		return () => {
			if (!asset) return;
			backgroundStore.setState((prev) => {
				if (!prev) return prev;
				if (prev.id !== asset.id) return prev;
				return null;
			}, true);
		};
	}, [asset, use]);
};
