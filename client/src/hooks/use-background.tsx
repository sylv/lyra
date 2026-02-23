import { useEffect } from "react";
import { create } from "zustand";
import type { ImageAsset } from "../components/image";

export const backgroundStore = create<ImageAsset | null>(() => null);

export const useDynamicBackground = (asset: ImageAsset | null, use?: boolean) => {
	useEffect(() => {
		if (use === false) return;
		backgroundStore.setState(asset, true);
		return () => {
			if (!asset) return;
			backgroundStore.setState((prev) => {
				if (!prev || prev.id !== asset.id) return prev;
				return null;
			}, true);
		};
	}, [asset, use]);
};
