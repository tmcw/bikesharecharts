<script>
	import { onMount } from 'svelte';
	import * as d3 from 'd3';
	export let context;
	export let id;
	export let domain;
	export let id_legend;
	export let station_information;
	let el;
	let margin = { top: 0, right: 0, bottom: 0, left: 0 };
	let height = 100;
	let width = 1400;

	const my_id = id_legend.get(id);
	let my_information = station_information.data.stations.find(
		(station) => station.station_id == my_id
	);

	onMount(async () => {
		let data = await context
			.query(`SELECT * FROM "station_status.parquet" WHERE station_ids = ${id};`)
			.then((r) =>
				r.toArray().map((r) => {
					return {
						...r,
						times: new Date(r.times)
					};
				})
			);
		let xScale = d3
			.scaleTime()
			.domain(domain)
			.range([margin.left, width - margin.right]);
		const keys = ['num_docks_available', 'num_ebikes_available', 'num_bikes_available'];
		const series = d3.stack().order(d3.stackOrderNone).offset(d3.stackOffsetExpand).keys(keys)(
			// distinct series keys, in input order
			data
		);
		let y = d3
			.scaleLinear()
			.domain([0, 1])
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

		svg.on('mousemove', function (e) {
			const [x, y] = d3.pointer(e, svg.node());
			const index = xScale.invert(x);
			console.log(x, index);
			const dIndex = data.findIndex((d) => d.times > index);
		});
		//00E893
	});
</script>

<!-- markup (zero or more items) goes here -->
<div class="row">
	<div class="info">
		<h3>{my_information.name}</h3>
	</div>
	<div bind:this={el} class="chart" />
</div>

<style>
	h3 {
		margin: 0;
		padding: 0;
		font-size: 12px;
	}
	.info {
		padding: 10px;
		background: #333;
	}
	.row {
		line-height: 1;
		display: grid;
		grid-template-columns: 150px 1fr;
		border-top: 10px solid black;
		border-bottom: 10px solid black;
	}
	.chart {
		background: #333;
	}
</style>
