import { NodeListFilter } from "@/components/nodes/node-list-filter";
import { DisplayKind, NodeList } from "@/components/nodes/node-list";
import { OrderBy } from "../@generated/gql/graphql";
import { useParams } from "react-router";

export function LibraryRoute() {
  const { libraryId } = useParams<{ libraryId: string }>();
  if (!libraryId) {
    return null;
  }
  return (
    <div className="mt-7">
      <NodeListFilter type="movies_series" defaultOrderBy={OrderBy.ReleasedAt} filterOverride={{ libraryId }}>
        {(filter) => (
          <div className="mt-3 flex flex-wrap gap-4">
            <div className="relative w-full">
              <div
                className="grid gap-4"
                style={{ gridTemplateColumns: "repeat(auto-fill, minmax(clamp(145px, 40vw, 174px), 1fr))" }}
              >
                <NodeList displayKind={DisplayKind.Poster} filter={filter} />
              </div>
            </div>
          </div>
        )}
      </NodeListFilter>
    </div>
  );
}
