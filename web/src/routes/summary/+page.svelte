<script lang="ts">
	import { onMount } from 'svelte';
	import * as duckdb from '@duckdb/duckdb-wasm';
	import duckdb_wasm from '@duckdb/duckdb-wasm/dist/duckdb-mvp.wasm?url';
	import mvp_worker from '@duckdb/duckdb-wasm/dist/duckdb-browser-mvp.worker.js?url';
	import duckdb_wasm_eh from '@duckdb/duckdb-wasm/dist/duckdb-eh.wasm?url';
	import eh_worker from '@duckdb/duckdb-wasm/dist/duckdb-browser-eh.worker.js?url';
	import Axis from '$lib/axis.svelte';
	import * as d3 from 'd3';
	import { rule } from '$lib/stores';

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

	async function init() {
		const bundle = await duckdb.selectBundle(MANUAL_BUNDLES);
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
		let domain = (
			await conn.query(
				`SELECT MIN(times) min, MAX(times) as max FROM "station_status.parquet" WHERE station_ids < 100;`
			)
		).toArray()[0];

		return {
			domain: [new Date(domain.min), new Date(domain.max)]
		};
	});

	let margin = { left: 0, right: 0, top: 0, bottom: 0 };
	let height = 700;
	let width = 1400;

	onMount(async () => {
		const conn = await context;
		const data = (
			await conn.query(
				`SELECT SUM(num_bikes_available)::integer as num_bikes_available, SUM(num_ebikes_available)::integer as num_ebikes_available, SUM(num_docks_available)::integer as num_docks_available, times FROM "station_status.parquet" GROUP BY times ORDER BY times;`
			)
		)
			.toArray()
			.map((row) => {
				return {
					...row.toJSON()
				};
			});

		console.log(data);

		let domain = (
			await conn.query(
				`SELECT MIN(times) min, MAX(times) as max FROM "station_status.parquet" WHERE station_ids < 100;`
			)
		).toArray()[0];
		let xScale = d3
			.scaleTime()
			.domain([new Date(domain.min), new Date(domain.max)])
			.range([margin.left, width - margin.right]);
		const keys = ['num_ebikes_available', 'num_bikes_available', 'num_docks_available'];
		const series = d3.stack().order(d3.stackOrderNone).keys(keys)(
			// distinct series keys, in input order
			data
		);
		let y = d3
			.scaleLinear()
			.domain([0, d3.max(series, (d) => d3.max(d, (d) => d[1])) as number])
			.range([height - margin.bottom, margin.top]);
		const area = d3
			.area()
			.curve(d3.curveStepAfter)
			.x((d) => {
				return xScale(d.data.times);
			})
			.y0((d) => {
				return y(d[0]);
			})
			.y1((d) => {
				return y(d[1]);
			});

		let svg = d3
			.select(el)
			.append('svg')
			.attr('class', 'chart')
			.attr('viewBox', [0, 0, width, height])
			.attr('width', width)
			.attr('height', height);
		const color = d3.scaleOrdinal().domain(keys).range(['#4CF6C3', '#0068FF', '#555']);

		svg
			.append('g')
			.selectAll()
			.data(series)
			.join('path')
			.attr('fill', (d) => color(d.key))
			.attr('d', (d) => {
				return area(d);
			})
			.append('title')
			.text((d) => d.key);

		const tooltip = svg.append('g');
		tooltip.append('rect').attr('width', 2).attr('height', height).attr('fill', '#FF433D');
		tooltip.append('text').attr('transform', `translate(10, 20)`).attr('fill', '#fff');

		svg.on('mousemove', function (e) {
			const [x, y] = d3.pointer(e, svg.node());
			const index = xScale.invert(x);
			rule.set([x, index]);
		});
		svg.on('mouseout', () => {
			rule.set(null);
		});

		rule.subscribe((ruleValue) => {
			if (ruleValue) {
				const [x, date] = ruleValue;
				const d = data.find((row) => row.times > date);
				tooltip.attr('transform', `translate(${x}, 0)`);
				tooltip
					.select('text')
					.text(`${d.num_bikes_available} bikes\n${d.num_ebikes_available} ebikes`);
			}
		});
	});
</script>

<div class="header">
	<div class="actionbox">
		<svg width="40" height="40">
			<circle r="20" cx="20" cy="20" fill="#555" />
		</svg>
	</div>

	{#await data then { domain: xDomain }}
		<Axis domain={xDomain} />
	{/await}
	<div class="sidebox">
		<div class="controls">
			<div class="legend">
				<div>
					<div class="legend-block" style="background: #4CF6C3; color: black;">EBikes</div>
				</div>
				<div>
					<div class="legend-block" style="background: #0068FF">Bikes</div>
				</div>
				<div>
					<div class="legend-block" style="background: #555">Docks</div>
				</div>
			</div>
		</div>
	</div>
	<div class="row">
		<div bind:this={el} class="chart" />
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
	.row {
		line-height: 1;
		display: grid;
		grid-template-columns: 150px 1fr;
		border-top: 5px solid black;
		border-bottom: 5px solid black;
	}
	:global(body) {
		color: white;
		margin: 0;
		font-family: 'IBM Plex Mono', monospace;
		background: #000;
	}
</style>
