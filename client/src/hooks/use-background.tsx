import { readFragment } from "gql.tada";
import { useEffect } from "react";
import { create } from "zustand";
import { ImageAssetFrag, type ImageAsset } from "../components/image";

export const backgroundStore = create<ImageAsset | null>(() => null);

export const useDynamicBackground = (asset: ImageAsset | null, use?: boolean) => {
	const assetId = asset ? readFragment(ImageAssetFrag, asset).id : null;

	useEffect(() => {
		if (use === false) return;
		backgroundStore.setState(asset, true);
		return () => {
			if (!assetId) return;
			backgroundStore.setState((prev) => {
				if (!prev) return prev;
				const resolvedPrev = readFragment(ImageAssetFrag, prev);
				if (resolvedPrev.id !== assetId) return prev;
				return null;
			}, true);
		};
	}, [asset, assetId, use]);
};
