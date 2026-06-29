/*
 * Conservative service worker for the Brenner theme.
 * Network-first so content is always fresh when online; falls back to the
 * runtime cache when offline, and to the cached home page for navigations.
 * Bump CACHE_VERSION to invalidate old caches on deploy.
 */
const CACHE_VERSION = "brenner-v1";
const OFFLINE_FALLBACK = "/";

self.addEventListener("install", (event) => {
	event.waitUntil(
		caches.open(CACHE_VERSION).then((cache) => cache.add(OFFLINE_FALLBACK)),
	);
	self.skipWaiting();
});

self.addEventListener("activate", (event) => {
	event.waitUntil(
		caches
			.keys()
			.then((keys) =>
				Promise.all(
					keys.filter((key) => key !== CACHE_VERSION).map((key) => caches.delete(key)),
				),
			)
			.then(() => self.clients.claim()),
	);
});

self.addEventListener("fetch", (event) => {
	const { request } = event;

	// Only handle same-origin GET requests.
	if (request.method !== "GET" || new URL(request.url).origin !== self.location.origin) {
		return;
	}

	event.respondWith(
		fetch(request)
			.then((response) => {
				// Cache a copy of successful responses for offline use.
				if (response && response.status === 200 && response.type === "basic") {
					const copy = response.clone();
					caches.open(CACHE_VERSION).then((cache) => cache.put(request, copy));
				}
				return response;
			})
			.catch(async () => {
				const cached = await caches.match(request);
				if (cached) return cached;
				if (request.mode === "navigate") {
					return caches.match(OFFLINE_FALLBACK);
				}
				return Response.error();
			}),
	);
});
