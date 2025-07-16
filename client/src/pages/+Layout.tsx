import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { httpLink } from "@trpc/client";
import { HomeIcon, Loader } from "lucide-react";
import type { FC, ReactNode } from "react";
import { Suspense, useMemo, useState } from "react";
import { IconText } from "../components/icon-text";
import { ThemeProvider } from "../components/theme-provider";
import {
	Sidebar,
	SidebarContent,
	SidebarGroup,
	SidebarGroupContent,
	SidebarGroupLabel,
	SidebarInset,
	SidebarMenu,
	SidebarMenuButton,
	SidebarMenuItem,
	SidebarProvider,
} from "../components/ui/sidebar";
import { Toaster } from "../components/ui/sonner";
import "./globals.css";
import { trpc } from "./trpc";
import { Player } from "../components/player/player";

// todo: use httpBatchLink once juno supports batching
export const Layout: FC<{ children: ReactNode }> = ({ children }) => {
	const [queryClient] = useState(() => new QueryClient());
	const [trpcClient] = useState(() =>
		trpc.createClient({
			links: [
				httpLink({
					url: "/api/trpc",
				}),
			],
		}),
	);

	return (
		<ThemeProvider>
			<trpc.Provider client={trpcClient} queryClient={queryClient}>
				<QueryClientProvider client={queryClient}>
					<SidebarProvider>
						<Sidebar>
							<SidebarContent>
								<SidebarGroup>
									<SidebarGroupLabel>Lyra</SidebarGroupLabel>
									<SidebarGroupContent>
										<SidebarMenu>
											<SidebarMenuItem>
												<SidebarMenuButton asChild>
													<a href="/">
														<HomeIcon />
														<span>Home</span>
													</a>
												</SidebarMenuButton>
											</SidebarMenuItem>
										</SidebarMenu>
									</SidebarGroupContent>
								</SidebarGroup>
							</SidebarContent>
						</Sidebar>
						<SidebarInset>
							<Suspense fallback={<Fallback />}>{children}</Suspense>
							<Toaster />
							<Player />
						</SidebarInset>
					</SidebarProvider>
				</QueryClientProvider>
			</trpc.Provider>
		</ThemeProvider>
	);
};

const LOADING_PHRASES = [
	"Science compels us to explode the sun",
	"Building terrain",
	"I wonder if it'll be friends with me? Hello ground!",
	"Oh no, not again",
	"Good Lord! What is happening in there?",
	"For external use only",
	"Scratch here to reveal prize",
	"Beats a hard kick in the face",
	"0100100001101001",
	"Made From 100% Recycled Pixels",
	"It's a feature, not a bug",
	"Quantum fluctuations detected...",
	"IS ANYONE THERE? OH - HI!",
	"I'm sorry, Dave. I'm afraid I can't do that.",
	"Oh, V...",
	"All right, David. Let's go. To the top, then",
	"Better Buckle Up!!",
	"GOOD YAKITORI NIGHT CITY",
	"Hypothesis: There can exist too much lava",
	"I'm doing stuff Lori. Thangs.",
	"I hear Nebraska's nice",
	"Doors and Corners, kid. That's where they get you.",
	"I think it must be damp",
	"I DECLARE BANKRUPTCY",
	"I'm not superstitious, but I'm a little stitious...",
	"I knew exactly what to do; but in a much more real sense, I had no idea what to do.",
	"K.I.S.S. Keep It Simple, Stupid. Great advice. Hurts my feelings every time.",
	"If you can't tell the difference, does it really matter?",
	"It doesn't look like anything to me.",
	"These violent delights have violent ends",
	"Wait a minute, this isn't my bedroom",
	"Oh Bojack, no. There is no other side. This is it.",
	"How many times have you seen this episode?",
	"Born amidst salt and smoke? Is he a ham?",
	"I have been falling FOR 30 MINUTES",
	"Yes yes, very sad. Anyway...",
	"You had one job. Just the one!",
	"You made those words up.",
	"Oh king of edible leaves, his majesty, the Spinach!",
	"Couple times? Are there Easter eggs you didn't get the first time?",
	"Yes, if it is to be said, so it be, so it is.",
	"You can't make a tomlette without breaking a few greggs",
	"3.6 Roentgen. Not great, not terrible.",
	"Oh snappers!",
	"I picked the wrong week to quit sniffing glue",
	"The egg bar is coveted as fuck",
	"Hey kids, what's for dinner?",
	"Am I livestock?",
	"The Work Is Mysterious And Important.",
	"Lana. LANA. LAAAANNNNAAA!!",
	"It is acceptable.",
	"I AM NOT CRAZY",
	"You think this is bad? This? This chicanery?",
	"He DEFECATED through a SUNROOF!",
	"Ahhhh, wire!",
	"Yeah, Mr. White! Yeah, science!",
	"No more half measures.",
	"I mean, it's one banana, Michael. What could it cost, $10?",
	"I don't understand the question and I won't respond to it",
	"There's always money in the banana stand",
	"Wadiyatalkinabeet?",
	"Yeah nah, yeah nah, yeah nah",
	"Just a couple of dimmies",
	"In and out, 20 minutes adventure",
	"Welcome to club, pal.",
];

const Fallback: FC = () => {
	const phrase = useMemo(() => {
		const index = Math.floor(Math.random() * LOADING_PHRASES.length);
		return LOADING_PHRASES[index];
	}, []);

	return <IconText icon={<Loader className="animate-spin" />} text={phrase} />;
};
