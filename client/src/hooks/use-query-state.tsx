import { useNavigate } from "@tanstack/react-router";
import { produce, type Draft } from "immer";
import { useCallback, useMemo, useState } from "react";
import z from "zod";
import { adaptQuerySchema } from "../lib/zod-forgiving";

interface UseQueryArgs<T extends z.ZodObject> {
	schema: T;
	overrides?: Partial<{ [K in keyof z.infer<T>]: z.infer<T>[K] | null | undefined }>;
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

export const useQueryState = <T extends z.ZodObject>({ schema, overrides }: UseQueryArgs<T>): UseQueryResult<T> => {
	const navigate = useNavigate<any>();
	const adaptedSchema = useMemo(() => adaptQuerySchema(schema), [schema]);

	const writeState = useCallback(
		(state: z.infer<T>) => {
			const data: Record<string, string | string[]> = {};
			for (const [key, value] of Object.entries(state)) {
				if (value == null) continue;
				data[key] = serialize(value);
			}

			console.log({ data });
			navigate({
				search: (old) =>
					({
						...old,
						...data,
					}) as any,
			});
		},
		[navigate, adaptedSchema],
	);

	const [state, setState] = useState(() => {
		const searchParams = new URLSearchParams(window.location.search);
		const parts: Record<string, unknown> = {};
		for (const [key, value] of searchParams.entries()) {
			if (parts[key]) {
				if (Array.isArray(parts[key])) parts[key].push(deserialize(value));
				else parts[key] = [parts[key], deserialize(value)];
			} else {
				parts[key] = deserialize(value);
			}
		}

		const parsed = adaptedSchema.safeParse({ ...parts, ...overrides });
		if (parsed.error) {
			console.error(parsed.error);
			throw new Error("Invalid useQueryState schema, it must be infallible");
		}

		writeState(parsed.data);
		return parsed.data;
	});

	const mutate = (producer: (prev: Draft<z.infer<T>>) => void) => {
		const nextState = produce(state, producer);
		setState(nextState);
		writeState(nextState);
	};

	return [state, mutate];
};
