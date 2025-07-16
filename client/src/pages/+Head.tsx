export const Head = () => {
	return (
		<script>
			{`
                let theme = localStorage.getItem("theme") || "system";
                if (theme === "system") {
                    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
                    theme = prefersDark ? "dark" : "light";
                }
                document.documentElement.classList.add(theme);
            `}
		</script>
	);
};
