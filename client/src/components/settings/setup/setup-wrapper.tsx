import { createContext, useCallback, useContext, useEffect, useRef, useState, type FC, type ReactNode } from "react";
import { useLocation, useNavigate } from "react-router";
import { refreshActiveQueries } from "../../../client";
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
  recheckSetup: () => Promise<void>;
  isRechecking: boolean;
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
  const location = useLocation();
  const navigate = useNavigate();
  const { pathname, search: searchStr, hash } = location;
  const request = useRef<Promise<void> | null>(null);
  const [data, setData] = useState<InitState | null>(null);
  const [error, setError] = useState<Error | null>(null);
  const [isRechecking, setIsRechecking] = useState(false);

  const pathnameRef = useRef(pathname);
  const searchStrRef = useRef(searchStr);
  const hashRef = useRef(hash);
  const dataRef = useRef<InitState | null>(null);

  pathnameRef.current = pathname;
  searchStrRef.current = searchStr;
  hashRef.current = hash;
  dataRef.current = data;

  const syncRoute = useCallback(
    (nextState: InitState) => {
      const currentPathname = pathnameRef.current;
      const currentSearchStr = searchStrRef.current;
      const currentHash = hashRef.current;

      if (isSetupReady(nextState)) {
        if (!isSetupPath(currentPathname)) {
          return;
        }

        navigate(getPreviousSetupRoute(currentSearchStr), { replace: true });
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

      navigate(target, { replace: true });
    },
    [navigate],
  );

  const syncSetupState = useCallback(
    async ({ recheck = false }: { recheck?: boolean } = {}) => {
      if (request.current) {
        return request.current;
      }

      if (recheck) {
        setIsRechecking(true);
      }

      request.current = fetchInitState(searchStrRef.current)
        .then(async (nextState) => {
          setError(null);

          const previousState = dataRef.current;
          const becameReady =
            recheck &&
            previousState != null &&
            !isSetupReady(previousState) &&
            isSetupReady(nextState) &&
            isSetupPath(pathnameRef.current);

          if (becameReady) {
            await refreshActiveQueries();
          }

          syncRoute(nextState);
          setData(nextState);
        })
        .catch((nextError) => {
          const error = nextError instanceof Error ? nextError : new Error("Failed to load setup state");
          setError(error);
          throw error;
        })
        .finally(() => {
          request.current = null;
          setIsRechecking(false);
        });

      return request.current;
    },
    [syncRoute],
  );

  const recheckSetup = useCallback(() => syncSetupState({ recheck: true }), [syncSetupState]);

  useEffect(() => {
    void syncSetupState().catch(() => {});
  }, [syncSetupState]);

  useEffect(() => {
    if (!data) return;
    syncRoute(data);
  }, [data, hash, navigate, pathname, searchStr]);

  const value = {
    state: data,
    recheckSetup,
    isRechecking,
  };

  if (error) {
    throw error;
  }

  if (!isSetupPath(pathname) && (!data || !isSetupReady(data))) {
    return <SetupContext.Provider value={value}>{null}</SetupContext.Provider>;
  }

  return <SetupContext.Provider value={value}>{children}</SetupContext.Provider>;
};
