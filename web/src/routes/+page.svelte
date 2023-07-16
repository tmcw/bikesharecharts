<script lang="ts">
	import StationRow from '$lib/station_row.svelte';
	import { getDuck } from '$lib/duckdb';
	import Axis from '$lib/axis.svelte';
	import { onMount } from 'svelte';
	import { rightWidth } from '$lib/stores';
	import { ExternalLink } from 'lucide-svelte';

	let el: HTMLDivElement;

	onMount(() => {
		const resizeObserver = new ResizeObserver((entries) => {
			for (const entry of entries) {
				rightWidth.set(entry.contentRect.width);
			}
		});
		resizeObserver.observe(el);
	});

	async function init() {
		const db = await getDuck();
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

		const count = (await conn.query(`SELECT count(*) as count FROM "station_status.parquet";`))
			.toArray()
			.map((row) => row.count);

		const station_ids = (
			await conn.query(
				`SELECT DISTINCT station_ids FROM "station_status.parquet" WHERE station_ids IN
				(SELECT id FROM station_information ORDER by capacity DESC);`
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
			count,
			station_information,
			station_ids,
			domain: [new Date(domain.min), new Date(domain.max)]
		};
	});
</script>

<div class="container">
	<div class="header">
		<div class="sidebox">
			<div class="hero">
				<h1>bikesharecharts</h1>
				<a href="https://github.com/tmcw/bikesharecharts" target="_blank" rel="noopener noreferrer"
					>GitHub
					<ExternalLink style="width:10px;vertical-align: middle;" />
				</a>
				{#await data then { station_information }}
					<input
						list="station-search-list"
						on:change={(event) => {
							const target = document.querySelector(`#station-${event.target.value}`);
							console.log(target);
							if (target) {
								target.scrollIntoView({
									block: 'center'
								});
								setTimeout(() => {
									target.classList.add('flash');
									setTimeout(() => {
										target.classList.remove('flash');
									}, 1000);
								}, 100);
							}
						}}
					/>
					<datalist id="station-search-list">
						{#each station_information.data.stations as station}
							<option value={station.station_id}>{station.name}</option>
						{/each}
					</datalist>
				{/await}
			</div>
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

		{#await data then { domain: xDomain }}
			<Axis domain={xDomain} />
		{/await}
	</div>

	<div class="stack" bind:this={el}>
		{#await data then { station_ids, domain: xDomain, id_legend, station_information }}
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
	.hero {
		padding: 10px;
		font-size: 12px;
	}
	.hero a {
		color: #fff;
		text-decoration: none;
	}
	.hero h1 {
		border: 1px solid #fff;
		display: inline-flex;
		font-size: inherit;
		margin: 0;
		padding: 5px;
	}
	.hero a:hover {
		text-decoration: underline;
	}
	.legend {
		padding: 10px;
		display: flex;
		align-items: center;
		gap: 10px;
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
		background: #000;
	}
	.header {
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
	:global(.flash) {
		filter: brightness(1.2) saturate(400%);
	}
	.stack {
		padding-right: 20px;
		border-top: 10px solid black;
		border-bottom: 10px solid black;
		line-height: 0;
	}
</style>
