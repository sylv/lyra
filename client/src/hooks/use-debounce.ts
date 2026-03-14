import { useEffect, useRef, useState } from "react";


export const useDebounce = <T>(
	value: T,
	delay: number,
	maxDelay?: number
): [T, boolean] => {
	const [debouncedValue, setDebouncedValue] = useState(value);
	const [isDebouncing, setIsDebouncing] = useState(false);
	const maxHandlerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

	useEffect(() => {
		setIsDebouncing(true);

		const commit = () => {
			setDebouncedValue(value);
			setIsDebouncing(false);
			if (maxHandlerRef.current !== null) {
				clearTimeout(maxHandlerRef.current);
				maxHandlerRef.current = null;
			}
		};

		const handler = setTimeout(commit, delay);

		if (maxDelay !== undefined && maxHandlerRef.current === null) {
			maxHandlerRef.current = setTimeout(commit, maxDelay);
		}

		return () => {
			clearTimeout(handler);
		};
	}, [value, delay, maxDelay]);

	return [debouncedValue, isDebouncing];
};
