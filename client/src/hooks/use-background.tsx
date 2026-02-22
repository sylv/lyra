import { useEffect } from "react";
import { create } from "zustand";
import type { ImageAsset } from "../components/image";

export const backgroundStore = create<ImageAsset | null>(() => null);

export const useDynamicBackground = (asset: ImageAsset | null, use?: boolean) => {
	useEffect(() => {
		if (!use) return;
		backgroundStore.setState(asset);
		return () => {
			if (!asset) return;
			backgroundStore.setState((prev) => {
				if (prev === asset) return null;
				return prev;
			});
		};
	}, [asset, use]);
};
