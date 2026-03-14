import { useLocation, useNavigate } from "@tanstack/react-router";
import { createContext, useContext, useEffect, useRef, useState, type FC, type ReactNode } from "react";
import { resetApolloClient } from "../../client";
import {
	fetchInitState,
	getPreviousSetupRoute,
	getRelativeLocationUri,
	getSetupRedirectUri,
	isSetupPath,
	isSetupReady,
	type InitState,
} from "./setup-state";

interface SetupContextValue {
	state: InitState | null;
	refresh: () => Promise<InitState>;
}

const SetupContext = createContext<SetupContextValue | null>(null);

export const useSetup = () => {
	const context = useContext(SetupContext);

	if (!context) {
		throw new Error("useSetup must be used within SetupWrapper");
	}

	return context;
};

export const SetupWrapper: FC<{ children: ReactNode }> = ({ children }) => {
	const navigate = useNavigate();
	const pathname = useLocation({
		select: (location) => location.pathname,
	});
	const searchStr = useLocation({
		select: (location) => location.searchStr,
	});
	const hash = useLocation({
		select: (location) => location.hash,
	});
	const request = useRef<Promise<InitState> | null>(null);
	const [data, setData] = useState<InitState | null>(null);
	const [error, setError] = useState<Error | null>(null);

	const pathnameRef = useRef(pathname);
	const searchStrRef = useRef(searchStr);
	const hashRef = useRef(hash);

	pathnameRef.current = pathname;
	searchStrRef.current = searchStr;
	hashRef.current = hash;

	const syncRoute = (nextState: InitState) => {
		const currentPathname = pathnameRef.current;
		const currentSearchStr = searchStrRef.current;
		const currentHash = hashRef.current;

		if (isSetupReady(nextState)) {
			if (!isSetupPath(currentPathname)) {
				return;
			}

			void navigate({ to: getPreviousSetupRoute(currentSearchStr) as never, replace: true });
			return;
		}

		const previous = isSetupPath(currentPathname)
			? getPreviousSetupRoute(currentSearchStr)
			: getRelativeLocationUri({
					pathname: currentPathname,
					searchStr: currentSearchStr,
					hash: currentHash,
				});
		const target = getSetupRedirectUri(nextState, previous);
		const currentUri = getRelativeLocationUri({
			pathname: currentPathname,
			searchStr: currentSearchStr,
			hash: currentHash,
		});

		if (currentUri === target) {
			return;
		}

		void navigate({ to: target as never, replace: true });
	};

	const refresh = async () => {
		if (request.current) {
			return request.current;
		}

		request.current = fetchInitState()
			.then((nextState) => {
				setError(null);

				if (isSetupReady(nextState) && isSetupPath(pathnameRef.current)) {
					// Recreate the Apollo client so any suspense/error state from the signed-out session is dropped.
					resetApolloClient();
				}

				setData(nextState);
				syncRoute(nextState);
				return nextState;
			})
			.catch((nextError) => {
				const error = nextError instanceof Error ? nextError : new Error("Failed to load setup state");
				setError(error);
				throw error;
			})
			.finally(() => {
				request.current = null;
			});

		return request.current;
	};

	useEffect(() => {
		void refresh().catch(() => {});
	}, []);

	useEffect(() => {
		if (!data) return;
		syncRoute(data);
	}, [data, hash, pathname, searchStr]);

	const value = {
		state: data,
		refresh,
	};

	if (error) {
		throw error;
	}

	if (!isSetupPath(pathname) && (!data || !isSetupReady(data))) {
		return <SetupContext.Provider value={value}>{null}</SetupContext.Provider>;
	}

	return <SetupContext.Provider value={value}>{children}</SetupContext.Provider>;
};
