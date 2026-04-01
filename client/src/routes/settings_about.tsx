import TmdbLogo from "../assets/tmdb-primary-short.svg";

export function SettingsAboutRoute() {
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
				className="mt-6 flex flex-col-reverse items-start gap-6 group md:flex-row md:items-center"
			>
				<img src={TmdbLogo} alt="TMDB Logo" className="h-8 shrink-0" />
				<div>
					<h3 className="text-sm group-hover:underline">Metadata sourced from TMDB</h3>
					<p className="text-xs text-zinc-400">
						This product uses the TMDB API but is not endorsed or certified by TMDB
					</p>
				</div>
			</a>
		</>
	);
}
