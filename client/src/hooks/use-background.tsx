import { useEffect } from "react";
import { create } from "zustand";

export const backgroundStore = create<string | null>(() => null);

export const useDynamicBackground = (url: string | null) => {
	useEffect(() => {
		backgroundStore.setState(url);
		return () => {
			if (!url) return;
			backgroundStore.setState((prev) => {
				if (prev === url) return null;
				return prev;
			});
		};
	}, [url]);
};
