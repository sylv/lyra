import { useQuery } from "@apollo/client";
import { graphql } from "gql.tada";
import { CornerUpLeft, Folder } from "lucide-react";
import { useState, useEffect, type FC, useMemo } from "react";

export interface DirectoryPickerProps {
	onPathChange: (path: string | null) => void;
	initialPath?: string;
}

const GET_FILES = graphql(`
    query GetFiles($path: String!) {
        listFiles(path: $path)
    }
`);

export const DirectoryPicker: FC<DirectoryPickerProps> = ({ onPathChange, initialPath = "/" }) => {
	const [currentPath, setCurrentPath] = useState<string>(initialPath);

	const { data, loading, error } = useQuery(GET_FILES, {
		variables: {
			path: currentPath,
		},
	});

	useEffect(() => {
		if (!loading && !error && data?.listFiles) {
			onPathChange(currentPath);
		} else if (!loading && error) {
			onPathChange(null);
		}
	}, [currentPath, loading, error, data, onPathChange]);

	const navigateToDirectory = (dirName: string) => {
		const newPath = currentPath === "/" ? `/${dirName}` : `${currentPath}/${dirName}`;
		setCurrentPath(newPath);
	};

	const navigateUp = () => {
		if (currentPath !== "/") {
			const parentPath = currentPath.substring(0, currentPath.lastIndexOf("/")) || "/";
			setCurrentPath(parentPath);
		}
	};

	const breadcrumbs = useMemo(() => {
		if (currentPath === "/") return [{ name: "Root", path: "/" }];

		const parts = currentPath.split("/").filter(Boolean);
		const breadcrumbs = [{ name: "Root", path: "/" }];

		let buildPath = "";
		for (const part of parts) {
			buildPath += `/${part}`;
			breadcrumbs.push({ name: part, path: buildPath });
		}

		return breadcrumbs;
	}, [currentPath]);

	return (
		<div className="border border-zinc-700 rounded-lg bg-zinc-950">
			<div className="flex items-center space-x-1 text-sm bg-zinc-900 border-b border-zinc-700 px-4 py-2 rounded-t-lg">
				{breadcrumbs.map((crumb, index) => (
					<div key={crumb.path} className="flex items-center">
						{index > 0 && <span className="mx-2 text-gray-400">/</span>}
						<button
							type="button"
							onClick={() => {
								setCurrentPath(crumb.path);
							}}
							className="text-white lowercase hover:underline"
						>
							{crumb.name}
						</button>
					</div>
				))}
			</div>

			{!loading && !error && data?.listFiles && (
				<div className="space-y-2 overflow-y-auto p-2 max-h-56">
					{currentPath !== "/" && (
						<button
							type="button"
							onClick={navigateUp}
							className="flex items-center w-full px-3 py-2 hover:bg-zinc-900 rounded-md text-sm"
						>
							<CornerUpLeft className="w-3.5 h-3.5 mr-2" />
							..
						</button>
					)}

					{data.listFiles.length === 0 ? (
						<div className="text-gray-500 text-sm italic p-2">No subdirectories found</div>
					) : (
						<ul className="space-y-1">
							{data.listFiles.map((dirName) => (
								<li key={dirName}>
									<button
										type="button"
										onClick={() => navigateToDirectory(dirName)}
										className="flex items-center w-full px-3 py-2 hover:bg-zinc-900 rounded-md text-sm"
									>
										<Folder className="w-4 h-4 mr-2 text-indigo-500" />
										{dirName}
									</button>
								</li>
							))}
						</ul>
					)}
				</div>
			)}
		</div>
	);
};
