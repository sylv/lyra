import { useEffect, useRef, type FC } from "react";
import { useCurrentValue } from "../hooks/use-current-value";

interface ViewLoaderProps {
	onLoadMore?: () => void;
}

export const ViewLoader: FC<ViewLoaderProps> = ({ onLoadMore }) => {
	const loaderRef = useRef<HTMLDivElement>(null);
	const loadMoreRef = useCurrentValue(() => onLoadMore);

	useEffect(() => {
		if (!loaderRef.current) return;

		const observer = new IntersectionObserver((entries) => {
			entries.forEach((entry) => {
				if (entry.isIntersecting) {
					loadMoreRef.current?.();
				}
			});
		});

		observer.observe(loaderRef.current);
		return () => {
			observer.disconnect();
		};
	}, [loaderRef.current]);

	return (
		<div className="relative w-full">
			<div ref={loaderRef} className="absolute bottom-0 left-0 right-0 h-[110vh] pointer-events-none z-10" />
		</div>
	);
};
