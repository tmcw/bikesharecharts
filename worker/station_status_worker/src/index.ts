/**
 * Welcome to Cloudflare Workers! This is your first scheduled worker.
 *
 * - Run `wrangler dev --local` in your terminal to start a development server
 * - Run `curl "http://localhost:8787/cdn-cgi/mf/scheduled"` to trigger the scheduled event
 * - Go back to the console to see what your worker has logged
 * - Update the Cron trigger in wrangler.toml (see https://developers.cloudflare.com/workers/wrangler/configuration/#triggers)
 * - Run `wrangler publish --name my-worker` to publish your worker
 *
 * Learn more at https://developers.cloudflare.com/workers/runtime-apis/scheduled-event/
 */

export interface Env {
	// Example binding to KV. Learn more at https://developers.cloudflare.com/workers/runtime-apis/kv/
	// MY_KV_NAMESPACE: KVNamespace;
	//
	// Example binding to Durable Object. Learn more at https://developers.cloudflare.com/workers/runtime-apis/durable-objects/
	// MY_DURABLE_OBJECT: DurableObjectNamespace;
	//
	// Example binding to R2. Learn more at https://developers.cloudflare.com/workers/runtime-apis/r2/
	DATA: R2Bucket;
}

export default {
	async scheduled(
		controller: ScheduledController,
		env: Env,
		ctx: ExecutionContext
	): Promise<void> {
                const res = await fetch('https://gbfs.citibikenyc.com/gbfs/en/station_status.json')
                if (!res.ok) {
                  throw new Error(`Endpoint down, status: ${res.status}`);
                }

                const compressionStream = new CompressionStream("gzip")
                const compressed = await new Response(
                  res.body?.pipeThrough(compressionStream)
                ).arrayBuffer()


                const time = new Date(controller.scheduledTime)

                const key = `station_status/${time.toISOString()}.json.gz`

                await env.DATA.put(key, compressed);
                console.log("OK: Scrape done");
	},
};
