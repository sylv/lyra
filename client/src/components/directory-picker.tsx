import { useQuery } from "@apollo/client/react";
import { CornerUpLeft, Folder } from "lucide-react";
import { useEffect, useState, type FC } from "react";
import { IconText } from "./icon-text";
import { Spinner } from "./ui/spinner";
import { graphql } from "../@generated/gql";

export interface DirectoryPickerProps {
	onPathChange: (path: string | null) => void;
	initialPath?: string;
}

const GetFiles = graphql(`
	query GetFiles($path: String!) {
		listFiles(path: $path)
	}
`);

export const DirectoryPicker: FC<DirectoryPickerProps> = ({ onPathChange, initialPath = "/" }) => {
	const [currentPath, setCurrentPath] = useState<string>(initialPath);
	const [pathInput, setPathInput] = useState<string>(initialPath);

	const { data, loading, error } = useQuery(GetFiles, {
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

	const normalizePath = (rawPath: string) => {
		const trimmed = rawPath.trim();
		if (!trimmed) return "/";

		const withLeadingSlash = trimmed.startsWith("/") ? trimmed : `/${trimmed}`;
		const normalized = withLeadingSlash.replace(/\/+/g, "/").replace(/\/$/, "");

		return normalized || "/";
	};

	const navigateToPath = (nextPath: string) => {
		const normalizedPath = normalizePath(nextPath);
		setCurrentPath(normalizedPath);
		setPathInput(normalizedPath);
	};

	const navigateToDirectory = (dirName: string) => {
		const newPath = currentPath === "/" ? `/${dirName}` : `${currentPath}/${dirName}`;
		navigateToPath(newPath);
	};

	const navigateUp = () => {
		if (currentPath !== "/") {
			const parentPath = currentPath.substring(0, currentPath.lastIndexOf("/")) || "/";
			navigateToPath(parentPath);
		}
	};

	useEffect(() => {
		const timeoutId = window.setTimeout(() => {
			const normalizedPath = normalizePath(pathInput);
			if (normalizedPath !== currentPath) {
				setCurrentPath(normalizedPath);
			}
		}, 300);

		return () => {
			window.clearTimeout(timeoutId);
		};
	}, [pathInput, currentPath]);

	const directories = data?.listFiles ?? [];

	return (
		<div className="rounded bg-black">
			<div className="bg-zinc-900 px-3 py-2 rounded-t-lg">
				<input
					type="text"
					value={pathInput}
					onChange={(event) => setPathInput(event.target.value)}
					placeholder="/path/to/library"
					className="w-full px-3 py-1.5 text-sm rounded-md outline-none"
				/>
			</div>

			<div className="h-56 space-y-2 overflow-y-auto">
				{loading ? (
					<div className="h-full w-full flex items-center justify-center p-3">
						<Spinner />
					</div>
				) : error || directories.length === 0 ? (
					<div className="h-full w-full flex items-center justify-center p-3">
						<IconText className="text-zinc-500" icon={<Folder className="size-4" />} text="I got nothin'" />
					</div>
				) : (
					<ul className="space-y-1">
						{currentPath !== "/" && (
							<button
								type="button"
								onClick={navigateUp}
								className="flex items-center w-full px-4 py-3 hover:bg-zinc-950 text-sm"
							>
								<CornerUpLeft className="size-3.5 mr-3" />
								..
							</button>
						)}
						{directories.map((dirName) => (
							<li key={dirName}>
								<button
									type="button"
									onClick={() => navigateToDirectory(dirName)}
									className="flex items-center w-full px-4 py-3 hover:bg-zinc-950 text-sm"
								>
									<Folder className="size-4 mr-3 text-indigo-500" />
									{dirName}
								</button>
							</li>
						))}
					</ul>
				)}
			</div>
		</div>
	);
};
