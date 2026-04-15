import useEmblaCarousel from "embla-carousel-react";
import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react";
import { useEffect, useState, type FC, type ReactNode } from "react";
import { cn } from "../lib/utils";

const CONTROL_CLASS_NAME =
	"flex size-5 shrink-0 items-center justify-center rounded border border-zinc-700/50 text-zinc-200 outline-none select-none transition-colors hover:bg-zinc-200/10 disabled:cursor-default disabled:opacity-40 disabled:hover:bg-transparent";

export const ShelfCarousel: FC<{
	title: ReactNode;
	children: ReactNode;
}> = ({ title, children }) => {
	const [emblaRef, emblaApi] = useEmblaCarousel({
		align: "start",
		containScroll: "trimSnaps",
		skipSnaps: true,
		slidesToScroll: 1,
		breakpoints: {
			"(min-width: 640px)": { slidesToScroll: 2 },
			"(min-width: 1024px)": { slidesToScroll: 4 },
			"(min-width: 1536px)": { slidesToScroll: 5 },
		},
	});
	const [canScrollPrev, setCanScrollPrev] = useState(false);
	const [canScrollNext, setCanScrollNext] = useState(false);
	const [scrollSnaps, setScrollSnaps] = useState<number[]>([]);
	const [selectedIndex, setSelectedIndex] = useState(0);

	useEffect(() => {
		if (!emblaApi) return;

		const syncScrollState = () => {
			setCanScrollPrev(emblaApi.canScrollPrev());
			setCanScrollNext(emblaApi.canScrollNext());
			setScrollSnaps(emblaApi.scrollSnapList());
			setSelectedIndex(emblaApi.selectedScrollSnap());
		};

		syncScrollState();
		emblaApi.on("select", syncScrollState);
		emblaApi.on("reInit", syncScrollState);

		return () => {
			emblaApi.off("select", syncScrollState);
			emblaApi.off("reInit", syncScrollState);
		};
	}, [emblaApi]);

	return (
		<section className="space-y-2">
			<div className="flex items-center justify-between gap-4">
				<div className="flex min-w-0 items-center gap-3">
					{title}
					{scrollSnaps.length > 1 ? (
						<div className="flex items-center gap-1.5">
							{scrollSnaps.map((_, index) => (
								<button
									type="button"
									key={index}
									className={cn(
										"h-2 w-2 rounded-full transition-colors",
										index === selectedIndex ? "bg-zinc-100" : "bg-zinc-600 hover:bg-zinc-400",
									)}
									aria-label={`Go to page ${index + 1}`}
									onClick={() => emblaApi?.scrollTo(index)}
								/>
							))}
						</div>
					) : null}
				</div>
				<div className="flex items-center gap-2">
					<button
						type="button"
						className={CONTROL_CLASS_NAME}
						disabled={!canScrollPrev}
						aria-label="Scroll left"
						onClick={() => emblaApi?.scrollPrev()}
					>
						<ChevronLeftIcon className="size-6" />
					</button>
					<button
						type="button"
						className={CONTROL_CLASS_NAME}
						disabled={!canScrollNext}
						aria-label="Scroll right"
						onClick={() => emblaApi?.scrollNext()}
					>
						<ChevronRightIcon className="size-6" />
					</button>
				</div>
			</div>
			<div className="min-w-0 overflow-hidden touch-pan-y select-none" ref={emblaRef}>
				<div className="flex gap-4">{children}</div>
			</div>
		</section>
	);
};
