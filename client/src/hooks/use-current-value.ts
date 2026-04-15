import { useEffect, useRef } from "react";

export const useCurrentValue = <T>(value: T | (() => T)) => {
  const ref = useRef<T>(typeof value === "function" ? (value as () => T)() : value);

  useEffect(() => {
    ref.current = typeof value === "function" ? (value as () => T)() : value;
  }, [value]);

  return ref;
};
