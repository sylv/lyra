import type { graphql } from "gql.tada";

export type MediaKind = ReturnType<typeof graphql.scalar<"MediaKind">>;

export type MediaFilter = ReturnType<typeof graphql.scalar<"MediaFilter">>;
