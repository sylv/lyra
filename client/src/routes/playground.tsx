import { useSuspenseQuery } from "@apollo/client/react";
import { graphql } from "../@generated/gql";
import { createFileRoute } from "@tanstack/react-router";
import { Navigate } from "@tanstack/react-router";
import { useState } from "react";
import { DirectoryPicker } from "@/components/directory-picker";
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

export const Route = createFileRoute("/playground")({
	component: PlaygroundRoute,
});

function PlaygroundRoute() {
	const [, setPath] = useState<string | null>("/");
	const { data } = useSuspenseQuery(PlaygroundViewerQuery);

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
