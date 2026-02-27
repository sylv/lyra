import { ApolloClient, InMemoryCache } from "@apollo/client";
import { HttpLink } from "@apollo/client/link/http";
import { relayStylePagination } from "@apollo/client/utilities";

export const client = new ApolloClient({
    link: new HttpLink({
        uri: "/api/graphql",
    }),
    cache: new InMemoryCache({
        typePolicies: {
            Query: {
                fields: {
                    rootList: relayStylePagination(["filter"]),
                    itemList: relayStylePagination(["filter"]),
                },
            },
        },
    }),
});