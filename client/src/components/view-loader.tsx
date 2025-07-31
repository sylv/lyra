import { useEffect, useRef, type FC } from "react";

interface ViewLoaderProps {
	onLoadMore?: () => void;
}

export const ViewLoader: FC<ViewLoaderProps> = ({ onLoadMore }) => {
	const loaderRef = useRef<HTMLDivElement>(null);

	useEffect(() => {
		if (!onLoadMore || !loaderRef.current) return;

		const observer = new IntersectionObserver((entries) => {
			entries.forEach((entry) => {
				if (entry.isIntersecting) {
					onLoadMore();
				}
			});
		});

		observer.observe(loaderRef.current);
		return () => {
			observer.disconnect();
		};
	}, [onLoadMore]);

	return (
		<div className="relative w-full">
			<div ref={loaderRef} className="absolute bottom-0 left-0 right-0 h-[110vh] pointer-events-none z-10" />
		</div>
	);
};
