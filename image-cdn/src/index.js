export default {
  async fetch(request, env, ctx) {
    try {
      const incomingUrl = new URL(request.url);
      const pathname = incomingUrl.pathname;
      const searchParams = incomingUrl.search;
      const backendBase = "https://iplustsolution-image-processing.hf.space";
      const targetUrl = `${backendBase}${pathname}${searchParams}`;
      const cache = caches.default;
      let response = await cache.match(request);
      if (response) {
        return response;
      }
      response = await fetch(targetUrl);
      if (!response.ok) {
        return new Response(`Failed to fetch image: ${response.status}`, { status: 500 });
      }
      const resToCache = new Response(response.body, response);
	    ctx.waitUntil(cache.put(request, resToCache.clone()));
      return resToCache;
    } catch (err) {
      return new Response("Internal Error: " + err.message, { status: 500 });
    }
  },
};