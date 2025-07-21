/* eslint-disable */
/* prettier-ignore */

export type introspection_types = {
    'Boolean': unknown;
    'File': { kind: 'OBJECT'; name: 'File'; fields: { 'backendName': { name: 'backendName'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'String'; ofType: null; }; } }; 'editionName': { name: 'editionName'; type: { kind: 'SCALAR'; name: 'String'; ofType: null; } }; 'id': { name: 'id'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null; }; } }; 'key': { name: 'key'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'String'; ofType: null; }; } }; 'pendingAutoMatch': { name: 'pendingAutoMatch'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null; }; } }; 'unavailableSince': { name: 'unavailableSince'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null; } }; }; };
    'Float': unknown;
    'Int': unknown;
    'Media': { kind: 'OBJECT'; name: 'Media'; fields: { 'backgroundUrl': { name: 'backgroundUrl'; type: { kind: 'SCALAR'; name: 'String'; ofType: null; } }; 'defaultConnection': { name: 'defaultConnection'; type: { kind: 'OBJECT'; name: 'File'; ofType: null; } }; 'description': { name: 'description'; type: { kind: 'SCALAR'; name: 'String'; ofType: null; } }; 'directConnections': { name: 'directConnections'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'LIST'; name: never; ofType: { kind: 'NON_NULL'; name: never; ofType: { kind: 'OBJECT'; name: 'File'; ofType: null; }; }; }; } }; 'endDate': { name: 'endDate'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null; } }; 'episodeNumber': { name: 'episodeNumber'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null; } }; 'id': { name: 'id'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null; }; } }; 'mediaType': { name: 'mediaType'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'ENUM'; name: 'MediaType'; ofType: null; }; } }; 'name': { name: 'name'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'String'; ofType: null; }; } }; 'parent': { name: 'parent'; type: { kind: 'OBJECT'; name: 'Media'; ofType: null; } }; 'parentId': { name: 'parentId'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null; } }; 'posterUrl': { name: 'posterUrl'; type: { kind: 'SCALAR'; name: 'String'; ofType: null; } }; 'rating': { name: 'rating'; type: { kind: 'SCALAR'; name: 'Float'; ofType: null; } }; 'runtimeMinutes': { name: 'runtimeMinutes'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null; } }; 'seasonNumber': { name: 'seasonNumber'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null; } }; 'seasons': { name: 'seasons'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'LIST'; name: never; ofType: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null; }; }; }; } }; 'startDate': { name: 'startDate'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null; } }; 'thumbnailUrl': { name: 'thumbnailUrl'; type: { kind: 'SCALAR'; name: 'String'; ofType: null; } }; 'tmdbItemId': { name: 'tmdbItemId'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null; }; } }; 'tmdbParentId': { name: 'tmdbParentId'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null; }; } }; }; };
    'MediaFilter': { kind: 'INPUT_OBJECT'; name: 'MediaFilter'; isOneOf: false; inputFields: [{ name: 'parentId'; type: { kind: 'SCALAR'; name: 'Int'; ofType: null; }; defaultValue: null }, { name: 'seasonNumbers'; type: { kind: 'LIST'; name: never; ofType: { kind: 'NON_NULL'; name: never; ofType: { kind: 'SCALAR'; name: 'Int'; ofType: null; }; }; }; defaultValue: null }, { name: 'search'; type: { kind: 'SCALAR'; name: 'String'; ofType: null; }; defaultValue: null }, { name: 'mediaTypes'; type: { kind: 'LIST'; name: never; ofType: { kind: 'NON_NULL'; name: never; ofType: { kind: 'ENUM'; name: 'MediaType'; ofType: null; }; }; }; defaultValue: null }]; };
    'MediaType': { name: 'MediaType'; enumValues: 'MOVIE' | 'SHOW' | 'EPISODE'; };
    'Query': { kind: 'OBJECT'; name: 'Query'; fields: { 'media': { name: 'media'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'OBJECT'; name: 'Media'; ofType: null; }; } }; 'mediaList': { name: 'mediaList'; type: { kind: 'NON_NULL'; name: never; ofType: { kind: 'LIST'; name: never; ofType: { kind: 'NON_NULL'; name: never; ofType: { kind: 'OBJECT'; name: 'Media'; ofType: null; }; }; }; } }; }; };
    'String': unknown;
};

/** An IntrospectionQuery representation of your schema.
 *
 * @remarks
 * This is an introspection of your schema saved as a file by GraphQLSP.
 * It will automatically be used by `gql.tada` to infer the types of your GraphQL documents.
 * If you need to reuse this data or update your `scalars`, update `tadaOutputLocation` to
 * instead save to a .ts instead of a .d.ts file.
 */
export type introspection = {
  name: never;
  query: 'Query';
  mutation: never;
  subscription: never;
  types: introspection_types;
};

import * as gqlTada from 'gql.tada';

declare module 'gql.tada' {
  interface setupSchema {
    introspection: introspection
  }
}