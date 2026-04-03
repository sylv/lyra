import useEmblaCarousel from "embla-carousel-react";
import { ArrowRightIcon, ChevronLeftIcon, ChevronRightIcon } from "lucide-react";
import { useEffect, useState, type FC } from "react";
import { Link } from "react-router";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import { getPathForCollection } from "../lib/getPathForMedia";
import { cn } from "../lib/utils";
import { NodePosterDetail } from "./nodes/node-poster-detail";

export const CollectionShelfFragment = graphql(`
	fragment CollectionShelf on Collection {
		id
		name
		itemCount
		nodeList(first: 12) {
			nodes {
				id
				...NodePoster
			}
			pageInfo {
				hasNextPage
			}
		}
	}
`);

const CONTROL_CLASS_NAME =
	"flex size-5 shrink-0 items-center justify-center rounded border border-zinc-700/50 text-zinc-200 outline-none select-none transition-colors hover:bg-zinc-200/10 disabled:cursor-default disabled:opacity-40 disabled:hover:bg-transparent";

export const CollectionShelf: FC<{
	collection: FragmentType<typeof CollectionShelfFragment>;
}> = ({ collection: collectionRaw }) => {
	const collection = unmask(CollectionShelfFragment, collectionRaw);
	if (collection.nodeList.nodes.length === 0) return null;
	const collectionPath = getPathForCollection(collection.id);
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
	const hasMore = collection.nodeList.pageInfo.hasNextPage;

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
					<Link to={collectionPath} className="truncate text-xl font-semibold hover:underline">
						{collection.name}
					</Link>
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
									aria-label={`Go to ${collection.name} page ${index + 1}`}
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
						aria-label={`Scroll ${collection.name} left`}
						onClick={() => emblaApi?.scrollPrev()}
					>
						<ChevronLeftIcon className="size-6" />
					</button>
					<button
						type="button"
						className={CONTROL_CLASS_NAME}
						disabled={!canScrollNext}
						aria-label={`Scroll ${collection.name} right`}
						onClick={() => emblaApi?.scrollNext()}
					>
						<ChevronRightIcon className="size-6" />
					</button>
				</div>
			</div>
			<div className="min-w-0 overflow-hidden touch-pan-y" ref={emblaRef}>
				<div className="flex gap-4">
					{collection.nodeList.nodes.map((node) => (
						<div className="min-w-0 flex-[0_0_9rem]" key={node.id}>
							<NodePosterDetail node={node} />
						</div>
					))}
					{hasMore ? (
						<div className="min-w-0 flex-[0_0_9rem]">
							<Link to={collectionPath} className="group flex h-full flex-col gap-2 outline-none">
								<div className="flex flex-col h-full justify-between rounded-sm border border-zinc-700/50 p-4 transition-colors">
									<div className="uppercase font-semibold text-[11px] text-zinc-500">{collection.name}</div>
									<div>
										<div className="text-sm font-semibold text-zinc-100 group-hover:underline">View More</div>
										<div className="mt-1 flex items-center gap-2 text-xs text-zinc-400">
											<div>{collection.itemCount} items</div>
											<ArrowRightIcon className="size-4 transition-transform group-hover:translate-x-0.5" />
										</div>
									</div>
								</div>
							</Link>
						</div>
					) : null}
				</div>
			</div>
		</section>
	);
};
