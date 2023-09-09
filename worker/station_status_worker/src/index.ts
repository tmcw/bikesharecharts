export interface Env {
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
