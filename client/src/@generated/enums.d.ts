import type { graphql } from "gql.tada";

export type MediaType = ReturnType<typeof graphql.scalar<"MediaType">>;

export type MediaFilter = ReturnType<typeof graphql.scalar<"MediaFilter">>;
