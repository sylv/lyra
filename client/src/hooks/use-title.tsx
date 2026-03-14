import { useEffect } from "react";

const defaultTitle = "Lyra";
const defaultSuffix = " — Lyra";

const resetTitle = () => {
	document.title = defaultTitle;
};

export const useTitle = (title?: string) => {
	useEffect(() => {
		if (!title) {
			resetTitle();
		} else {
			document.title = title + defaultSuffix;
			return resetTitle;
		}
	}, [title]);
};
