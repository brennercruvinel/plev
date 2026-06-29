// Color palette switcher
//
// Applies a `data-palette` attribute on <html>, orthogonal to `data-theme`
// (light/dark). The server may render a default palette; user choices are
// persisted in localStorage["palette"]. The "default" id means "no palette
// override" (the orange accent from accent_color/_dark), so it removes the
// attribute instead of setting it.
(function () {
	const DEFAULT_ID = "default";

	// Palette rendered server-side (config.extra.default_palette), if any.
	const serverPalette = document.documentElement.getAttribute("data-palette");
	if (serverPalette) {
		document.documentElement.setAttribute("data-default-palette", serverPalette);
	}

	function applyPalette(id) {
		if (!id || id === DEFAULT_ID) {
			document.documentElement.removeAttribute("data-palette");
		} else {
			document.documentElement.setAttribute("data-palette", id);
		}
	}

	// Restore the stored choice as early as possible.
	const storedPalette = localStorage.getItem("palette");
	if (storedPalette) {
		applyPalette(storedPalette);
	}

	function currentPalette() {
		return (
			localStorage.getItem("palette") ||
			serverPalette ||
			DEFAULT_ID
		);
	}

	function updateActiveButton(id) {
		document.querySelectorAll("#theme-switcher button[data-palette-id]").forEach((button) => {
			button.classList.toggle(
				"active",
				button.getAttribute("data-palette-id") === id
			);
		});
	}

	function switchPalette(id) {
		applyPalette(id);
		if (id && id !== DEFAULT_ID) {
			localStorage.setItem("palette", id);
		} else {
			localStorage.removeItem("palette");
		}
		updateActiveButton(id);
	}

	document.querySelectorAll("#theme-switcher button[data-palette-id]").forEach((button) => {
		button.addEventListener("click", function () {
			switchPalette(button.getAttribute("data-palette-id"));
		});
	});

	// Reflect the active palette on load.
	updateActiveButton(currentPalette());

	// Expose for parity with the theme switcher.
	window.switchPalette = switchPalette;
})();
