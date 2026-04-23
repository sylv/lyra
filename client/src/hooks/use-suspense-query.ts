import { useMemo } from "react";
import {
  useQuery,
  type AnyVariables,
  type OperationContext,
  type UseQueryArgs,
  type UseQueryResponse,
} from "urql";

type UseSuspenseQueryArgs<Variables extends AnyVariables = AnyVariables, Data = unknown> = Omit<
  UseQueryArgs<Variables, Data>,
  "context"
> & {
  context?: Partial<OperationContext>;
};

export const useSuspenseQuery = <Data = unknown, Variables extends AnyVariables = AnyVariables>({
  context,
  ...args
}: UseSuspenseQueryArgs<Variables, Data>): UseQueryResponse<Data, Variables> => {
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

  return useQuery<Data, Variables>(queryArgs);
};
