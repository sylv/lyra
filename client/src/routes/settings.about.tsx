import { createFileRoute } from "@tanstack/react-router";
import TmdbLogo from "../assets/tmdb-primary-short.svg";

export const Route = createFileRoute("/settings/about")({
	component: RouteComponent,
});

function RouteComponent() {
	const buildDate = new Date(__BUILD_TIME__).toLocaleString();

	return (
		<>
			<div>
				<h3>Build</h3>
				<p className="text-sm text-zinc-400">
					{__BRANCH__} {__REVISION__}, built {buildDate}.
				</p>
			</div>
			<a
				href="https://www.themoviedb.org/"
				target="_blank"
				rel="noopener noreferrer"
				className="mt-6 flex items-center gap-6 group"
			>
				<div>
					<img src={TmdbLogo} alt="TMDB Logo" className="h-8" />
				</div>
				<div>
					<h3 className="group-hover:underline">Metadata sourced from TMDB</h3>
					<p className="text-zinc-400 text-sm">
						This product uses the TMDB API but is not endorsed or certified by TMDB
					</p>
				</div>
			</a>
		</>
	);
}
