import { useApolloClient } from "@apollo/client/react";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useRef, useState, type FC, type ReactNode } from "react";
import { SetupModal, type InitState } from "./setup-modal";

export const SetupWrapper: FC<{ children: ReactNode }> = ({ children }) => {
	const fetching = useRef(false);
	const client = useApolloClient();
	const navigate = useNavigate();
	const [data, setData] = useState<InitState | null>(null);
	const [showModal, setShowModal] = useState(false);

	const fetchData = async () => {
		if (fetching.current) return;
		fetching.current = true;

		try {
			const res = await fetch("/api/init");
			const data = await res.json();
			setData(data);
			return data;
		} finally {
			fetching.current = false;
		}
	};

	useEffect(() => {
		fetchData();
	}, []);

	useEffect(() => {
		if (!data) return;
		if (data.state === "ready") {
			if (showModal) {
				setShowModal(false);
				client.refetchQueries({ include: "all" });
			}
		} else {
			setShowModal(true);
			navigate({ to: "/" });
		}
	}, [client, data, navigate, showModal]);

	const mutate = async () => {
		await fetchData();
	};

	return (
		<>
			{children}
			{showModal && data && <SetupModal state={data} mutate={mutate} />}
		</>
	);
};
