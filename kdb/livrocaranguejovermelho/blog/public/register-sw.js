// Register the service worker for offline support and installability.
if ("serviceWorker" in navigator) {
	window.addEventListener("load", () => {
		navigator.serviceWorker.register("/sw.js").catch((error) => {
			console.error("[pwa] service worker registration failed:", error);
		});
	});
}
