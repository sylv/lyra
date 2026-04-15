import { useMemo } from "react";
import {
  useQuery,
  type AnyVariables,
  type OperationContext,
  type UseQueryArgs,
  type UseQueryResponse,
  type UseQueryState,
} from "urql";

type UseSuspenseQueryArgs<Variables extends AnyVariables = AnyVariables, Data = unknown> = Omit<
  UseQueryArgs<Variables, Data>,
  "context"
> & {
  context?: Partial<OperationContext>;
};

type UseSuspenseQueryState<Data, Variables extends AnyVariables> = Omit<UseQueryState<Data, Variables>, "data"> & {
  data: Data;
};

type UseSuspenseQueryResponse<Data, Variables extends AnyVariables> = [
  UseSuspenseQueryState<Data, Variables>,
  UseQueryResponse<Data, Variables>[1],
];

export const useSuspenseQuery = <Data = unknown, Variables extends AnyVariables = AnyVariables>({
  context,
  ...args
}: UseSuspenseQueryArgs<Variables, Data>): UseSuspenseQueryResponse<Data, Variables> => {
  const queryContext = useMemo(
    () => ({
      ...context,
      suspense: true,
    }),
    [context],
  );

  const queryArgs = {
    ...args,
    context: queryContext,
  } as UseQueryArgs<Variables, Data>;

  return useQuery<Data, Variables>(queryArgs) as UseSuspenseQueryResponse<Data, Variables>;
};
