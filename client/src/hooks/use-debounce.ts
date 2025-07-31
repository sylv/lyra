import { useEffect, useState } from "react";

export const useDebounce = <T>(value: T, delay: number): [T, boolean] => {
	const [debouncedValue, setDebouncedValue] = useState(value);
	const [isDebouncing, setIsDebouncing] = useState(false);

	useEffect(() => {
		setIsDebouncing(true);
		const handler = setTimeout(() => {
			setDebouncedValue(value);
			setIsDebouncing(false);
		}, delay);

		return () => {
			clearTimeout(handler);
		};
	}, [value, delay]);

	return [debouncedValue, isDebouncing];
};
