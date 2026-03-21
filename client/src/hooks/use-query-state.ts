import { useLocation, useNavigate } from "@tanstack/react-router";

export const useQueryState = <T>(key: string, defaultValue?: T): [T, (value: T) => void] => {
	const location = useLocation();
	const navigate = useNavigate();
	const searchParams = new URLSearchParams(location.searchStr);

	const rawValue = searchParams.get(key);
	const value = rawValue ? JSON.parse(atob(rawValue)) : defaultValue;

	const setValue = (value: T) => {
		const url = new URL(window.location.href);
		const stringified = JSON.stringify(value);

		if (JSON.stringify(defaultValue) === stringified) {
			url.searchParams.delete(key);
		} else {
			url.searchParams.set(key, btoa(stringified));
		}

		navigate({ to: `${url.pathname}${url.search}${url.hash}` });
	};

	return [value, setValue];
};
