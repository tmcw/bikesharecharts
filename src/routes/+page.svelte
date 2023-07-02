<script lang="ts">
	import { onMount } from 'svelte';
	import * as duckdb from '@duckdb/duckdb-wasm';
	import duckdb_wasm from '@duckdb/duckdb-wasm/dist/duckdb-mvp.wasm?url';
	import mvp_worker from '@duckdb/duckdb-wasm/dist/duckdb-browser-mvp.worker.js?url';
	import duckdb_wasm_eh from '@duckdb/duckdb-wasm/dist/duckdb-eh.wasm?url';
	import eh_worker from '@duckdb/duckdb-wasm/dist/duckdb-browser-eh.worker.js?url';
	import StationRow from '../lib/station_row.svelte';
	import Axis from '../lib/axis.svelte';

	export const ssr = false;

	const MANUAL_BUNDLES: duckdb.DuckDBBundles = {
		mvp: {
			mainModule: duckdb_wasm,
			mainWorker: mvp_worker
		},
		eh: {
			mainModule: duckdb_wasm_eh,
			mainWorker: eh_worker
		}
	};

	async function init() {
		// Select a bundle based on browser checks
		const bundle = await duckdb.selectBundle(MANUAL_BUNDLES);
		// Instantiate the asynchronus version of DuckDB-wasm
		const worker = new Worker(bundle.mainWorker!);
		const logger = new duckdb.ConsoleLogger();
		const db = new duckdb.AsyncDuckDB(logger, worker);
		await db.instantiate(bundle.mainModule, bundle.pthreadWorker);

		const res = await fetch('./data.parquet');
		await db.registerFileBuffer('station_status.parquet', new Uint8Array(await res.arrayBuffer()));

		const conn = await db.connect();

		return conn;
	}

	let context = init();

	let data = context.then(async (conn) => {
		const id_legend_raw = await fetch('./id_map.json').then((r) => r.json());

		const id_legend = new Map(Object.entries(id_legend_raw).map(([k, v]) => [v, k]));

		const station_information = await fetch('./station_information.json').then((r) => r.json());

		const station_ids = (
			await conn.query(
				`SELECT DISTINCT station_ids FROM "station_status.parquet" WHERE station_ids < 10;`
			)
		)
			.toArray()
			.map((row) => row.station_ids);
		let domain = (
			await conn.query(
				`SELECT MIN(times) min, MAX(times) as max FROM "station_status.parquet" WHERE station_ids < 100;`
			)
		).toArray()[0];

		return {
			id_legend,
			station_information,
			station_ids,
			domain: [new Date(domain.min), new Date(domain.max)]
		};
	});
</script>

{#await data then { station_ids, domain: xDomain, id_legend, station_information }}
	<div class="header">
		<div class="actionbox">
			<svg width="40" height="40">
				<circle r="20" cx="20" cy="20" fill="#555" />
			</svg>
		</div>

		<Axis domain={xDomain} />
		<div class="sidebox">
			<div class="controls">
				<div class="legend">
					<div>
						<div class="legend-block" style="background: #4CF6C3; color: black;">Bikes</div>
					</div>
					<div>
						<div class="legend-block" style="background: #0068FF">EBikes</div>
					</div>
					<div>
						<div class="legend-block" style="background: #555">Docks</div>
					</div>
				</div>
			</div>
		</div>
		<div class="stack">
			{#await context}
				<p>loading…</p>
			{:then context}
				{#each station_ids as id}
					<StationRow domain={xDomain} {id_legend} {station_information} {context} {id} />
				{/each}
			{:catch error}
				<p style="color: red">{error.message}</p>
			{/await}
		</div>
	</div>
{/await}

<style>
	.legend {
		padding: 20px;
	}
	.legend > div {
		padding-bottom: 5px;
	}
	.legend-block {
		display: inline-block;
		font-size: 12px;
		border-bottom: 1px solid #fff;
		padding: 2px 4px;
	}
	.actionbox {
		padding: 10px;
	}
	/* https://www.color-hex.com/color-palette/111776 */
	.header {
		display: grid;
		grid-template-columns: 150px 1fr;
		position: sticky;
		top: 0;
		background: #000;
	}
	:global(body) {
		color: white;
		margin: 0;
		font-family: 'IBM Plex Mono', monospace;
		background: #000;
	}
	.stack {
		padding-right: 20px;
		border-top: 10px solid black;
		border-bottom: 10px solid black;
		line-height: 0;
	}
</style>
