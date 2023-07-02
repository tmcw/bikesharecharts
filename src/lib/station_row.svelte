<script>
	import { onMount } from 'svelte';
	import * as d3 from 'd3';
	import { rule } from '$lib/stores';
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
		const keys = [
			'num_bikes_disabled',
			'num_ebikes_available',
			'num_bikes_available',
			'num_docks_available'
		];
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
			.attr('class', 'chart')
			.attr('viewBox', [0, 0, width, height])
			.attr('width', width)
			.attr('height', height);
		const color = d3.scaleOrdinal().domain(keys).range(['#FF423D', '#4CF6C3', '#0068FF', '#555']);

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
			document.querySelectorAll('svg.chart').forEach((el) => {
				if (el !== e.target) {
				}
				//	el.dispatchEvent(new MouseEvent('mousemove', e));
				//}
			});
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
					.text(
						`${d.num_bikes_available} bikes\n${d.num_ebikes_available} ebikes, ${d.num_docks_available} docks, ${d.num_bikes_disabled} disabled`
					);
			}
		});
	});
</script>

<!-- markup (zero or more items) goes here -->
<div class="row">
	<div class="info">
		<h3>{my_information.name}</h3>
		<div>capacity: {my_information.capacity}</div>
	</div>
	<div bind:this={el} class="chart" />
</div>

<style>
	h3 {
		margin: 0;
		padding: 0;
	}
	.info {
		font-size: 12px;
		padding: 10px;
		background: #333;
	}
	.row {
		line-height: 1;
		display: grid;
		grid-template-columns: 150px 1fr;
		border-top: 5px solid black;
		border-bottom: 5px solid black;
	}
	.chart {
		background: #333;
	}
</style>
