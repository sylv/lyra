import { usePageContext } from "vike-react/usePageContext";
import { navigate } from "vike/client/router";

export const useQueryState = <T>(key: string, defaultValue?: T): [T, (value: T) => void] => {
	const pageContext = usePageContext();

	const rawValue = pageContext.urlParsed.search[key];
	const value = rawValue ? JSON.parse(atob(rawValue)) : defaultValue;

	const setValue = (value: T) => {
		const url = new URL(window.location.href);
		const stringified = JSON.stringify(value);

		if (JSON.stringify(defaultValue) === stringified) {
			url.searchParams.delete(key);
		} else {
			url.searchParams.set(key, btoa(stringified));
		}

		navigate(url.toString());
	};

	return [value, setValue];
};
