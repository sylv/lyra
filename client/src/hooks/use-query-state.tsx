import { produce, type Draft } from "immer";
import { useCallback, useEffect, useState } from "react";
import { useLocation, useNavigate } from "react-router";
import z from "zod";
import { adaptQuerySchema } from "../lib/zod-forgiving";
import { useCurrentValue } from "./use-current-value";

interface UseQueryArgs<T extends z.ZodObject> {
  schema: T;
}

export type UseQueryResult<T extends z.ZodObject> = [z.infer<T>, (producer: (prev: Draft<z.infer<T>>) => void) => void];

const serialize = (value: unknown): string => {
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return "b64j" + btoa(JSON.stringify(value));
};

const deserialize = (value: string): unknown => {
  // we patch the schema for things like string to number coercion, so we only really have to handle JSON here
  if (value.startsWith("b64j")) {
    const json = atob(value.slice(4));
    try {
      return JSON.parse(json);
    } catch {}
  }

  return value;
};

export const useQueryState = <T extends z.ZodObject>(props: UseQueryArgs<T>): UseQueryResult<T> => {
  const location = useLocation();
  const navigate = useNavigate();
  const schema = useCurrentValue(() => adaptQuerySchema(props.schema));
  const parseSearch = useCallback((search: string) => {
    const searchParams = new URLSearchParams(search);
    const parts: Record<string, unknown> = {};
    for (const [key, value] of searchParams.entries()) {
      if (parts[key]) {
        if (Array.isArray(parts[key])) parts[key].push(deserialize(value));
        else parts[key] = [parts[key], deserialize(value)];
      } else {
        parts[key] = deserialize(value);
      }
    }

    const parsed = schema.current.safeParse(parts);
    if (parsed.error) {
      console.error(parsed.error);
      throw new Error("Invalid useQueryState schema, it must be infallible");
    }

    return parsed.data;
  }, []);

  const [state, setState] = useState(() => parseSearch(location.search));

  useEffect(() => {
    setState(parseSearch(location.search));
  }, [location.search, parseSearch]);

  const writeState = useCallback(
    (state: z.infer<T>) => {
      const data = new URLSearchParams();
      for (const [key, value] of Object.entries(state)) {
        if (value == null) continue;
        const values = Array.isArray(value) ? value.map(serialize) : [serialize(value)];
        for (const item of values) {
          data.append(key, item);
        }
      }

      navigate(
        {
          pathname: location.pathname,
          search: data.toString() ? `?${data.toString()}` : "",
          hash: location.hash,
        },
        { replace: true },
      );
    },
    [location.hash, location.pathname, navigate],
  );

  const mutate = (producer: (prev: Draft<z.infer<T>>) => void) => {
    const nextState = produce(state, producer);
    setState(nextState);
    writeState(nextState);
  };

  return [state, mutate];
};
