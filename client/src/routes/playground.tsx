import { useState } from "react";
import { Navigate } from "react-router";
import { graphql } from "../@generated/gql";
import { DirectoryPicker } from "@/components/directory-picker";
import { useSuspenseQuery } from "../hooks/use-suspense-query";
import { useTitle } from "../hooks/use-title";
import { ADMIN_BIT } from "../lib/user-permissions";

const PlaygroundViewerQuery = graphql(`
  query PlaygroundViewer {
    viewer {
      id
      permissions
    }
  }
`);

export function PlaygroundRoute() {
  const [, setPath] = useState<string | null>("/");
  const [{ data }] = useSuspenseQuery({ query: PlaygroundViewerQuery });

  useTitle("Playground");

  if ((data.viewer?.permissions ?? 0) & ADMIN_BIT) {
    return (
      <div className="p-6">
        <DirectoryPicker onPathChange={setPath} />
      </div>
    );
  }

  return <Navigate to="/" replace />;
}
