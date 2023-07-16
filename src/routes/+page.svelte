<script lang="ts">
	import * as duckdb from '@duckdb/duckdb-wasm';
	import duckdb_wasm from '@duckdb/duckdb-wasm/dist/duckdb-mvp.wasm?url';
	import mvp_worker from '@duckdb/duckdb-wasm/dist/duckdb-browser-mvp.worker.js?url';
	import duckdb_wasm_eh from '@duckdb/duckdb-wasm/dist/duckdb-eh.wasm?url';
	import eh_worker from '@duckdb/duckdb-wasm/dist/duckdb-browser-eh.worker.js?url';
	import StationRow from '$lib/station_row.svelte';
	import Axis from '$lib/axis.svelte';
	import { onMount } from 'svelte';
	import { rightWidth } from '$lib/stores';

	export const ssr = false;

	let el: HTMLDivElement;

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

	onMount(() => {
		const resizeObserver = new ResizeObserver((entries) => {
			for (const entry of entries) {
				rightWidth.set(entry.contentRect.width);
			}
		});
		console.log(el);
		resizeObserver.observe(el);
	});

	async function init() {
		// Select a bundle based on browser checks
		const bundle = await duckdb.selectBundle(MANUAL_BUNDLES);
		// Instantiate the asynchronus version of DuckDB-wasm
		const worker = new Worker(bundle.mainWorker!);
		const logger = new duckdb.ConsoleLogger(duckdb.LogLevel.ERROR);
		const db = new duckdb.AsyncDuckDB(logger, worker);
		await db.instantiate(bundle.mainModule, bundle.pthreadWorker);

		const res = await fetch('https://data.bikesharecharts.com/data.parquet');
		await db.registerFileBuffer('station_status.parquet', new Uint8Array(await res.arrayBuffer()));

		const info = await fetch('./station_information_array.json').then((r) => r.json());
		const id_legend = await fetch('./id_map.json').then((r) => r.json());
		const conn = await db.connect();

		await conn.query(
			`CREATE TABLE station_information(id VARCHAR, name VARCHAR, lon DOUBLE, lat DOUBLE, station_id VARCHAR, capacity INTEGER)`
		);

		for (let row of info) {
			let id = id_legend[row.station_id];

			if (id === undefined) {
				continue;
			}
			await conn.query(`
			INSERT INTO station_information(id, name, lat, lon, station_id, capacity)
			VALUES (${id}, '${row.name.replace(/'/g, `''`)}', ${row.lat}, ${row.lon}, '${row.station_id}', ${
				row.capacity
			})
			`);
		}

		return conn;
	}

	let context = init();

	let data = context.then(async (conn) => {
		const id_legend_raw = await fetch('./id_map.json').then((r) => r.json());

		const id_legend = new Map(Object.entries(id_legend_raw).map(([k, v]) => [v, k]));

		const station_information = await fetch('./station_information.json').then((r) => r.json());

		const bbox = [-74.019924, 40.669, -74.002422, 40.680141];

		/*
		const station_ids = (
			await conn.query(
				`SELECT DISTINCT station_ids FROM "station_status.parquet" WHERE station_ids IN
				
				(SELECT id FROM station_information
		WHERE lon > ${bbox[0]} AND lon < ${bbox[2]}
		AND lat > ${bbox[1]} AND lat < ${bbox[3]});`
			)
		)
			.toArray()
			.map((row) => row.station_ids);
			*/

		const count = (await conn.query(`SELECT count(*) as count FROM "station_status.parquet";`))
			.toArray()
			.map((row) => row.count);

		const station_ids = (
			await conn.query(
				`SELECT DISTINCT station_ids FROM "station_status.parquet" WHERE station_ids IN
				
				(SELECT id FROM station_information ORDER by capacity  DESC LIMIT 10);`
			)
		)
			.toArray()
			.map((row) => row.station_ids);

		/*

		const station_ids = (
			await conn.query(
				`SELECT DISTINCT station_ids, MEAN(num_ebikes_available / (num_ebikes_available + num_bikes_available)) d FROM "station_status.parquet"
				GROUP BY station_ids
				ORDER BY d DESC
				 LIMIT 20;`
			)
		)
			.toArray()
			.map((row) => row.station_ids);
			*/

		let domain = (
			await conn.query(
				`SELECT MIN(times) min, MAX(times) as max FROM "station_status.parquet" WHERE station_ids < 100;`
			)
		).toArray()[0];

		return {
			id_legend,
			count,
			station_information,
			station_ids,
			domain: [new Date(domain.min), new Date(domain.max)]
		};
	});
</script>

<div class="container">
	<div class="actionbox">
		<svg width="10" height="10">
			<circle r="5" cx="5" cy="5" fill="#555" />
		</svg>
	</div>

	<div class="header">
		{#await data then { domain: xDomain }}
			<Axis domain={xDomain} />
		{/await}
	</div>
	<div class="sidebox">
		<div class="controls">
			<div class="legend">
				<div>
					<div class="legend-block" style="background: #4CF6C3; color: black;">EBikes</div>
				</div>
				<div>
					<div class="legend-block" style="background: #FF423D">Disabled</div>
				</div>
				<div>
					<div class="legend-block" style="background: #0068FF">Bikes</div>
				</div>
				<div>
					<div class="legend-block" style="background: #555">Docks</div>
				</div>
				<div>
					<div class="legend-block" style="background: yellow; color: black;">Sun</div>
				</div>
			</div>
		</div>
	</div>
	<div class="stack" bind:this={el}>
		{#await data then { station_ids, domain: xDomain, id_legend, station_information, count }}
			{#await context}
				<p>loading…</p>
			{:then context}
				{#each station_ids as id}
					<StationRow domain={xDomain} {id_legend} {station_information} {context} {id} />
				{/each}
			{:catch error}
				<p style="color: red">{error.message}</p>
			{/await}
		{/await}
	</div>
</div>

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
	/* https://www.color-hex.com/color-palette/111776 */
	.container {
		display: grid;
		grid-template-columns: 150px 1fr;
		top: 0;
		background: #000;
	}
	.header {
		position: sticky;
		line-height: 0;
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
