<script lang="ts">
	import { onMount } from 'svelte';
	import * as d3 from 'd3';
	import { rule } from '$lib/stores';
	import type duckdb from '@duckdb/duckdb-wasm';
	import { rightWidth } from '$lib/stores';

	export let context: duckdb.AsyncDuckDBConnection;
	export let id: string;
	export let domain: [Date, Date];
	export let id_legend;
	export let station_information;

	let el: HTMLDivElement;
	let margin = { top: 0, right: 0, bottom: 0, left: 0 };
	let height = 100;
	let intersecting = false;
	let rendered = false;

	const my_id = id_legend.get(id);
	let my_information = station_information.data.stations.find(
		(station) => station.station_id == my_id
	);

	onMount(() => {
		const observer = new IntersectionObserver((entries) => {
			intersecting = entries[0].isIntersecting;
			if (intersecting) {
				if (rendered === false) {
					rendered = true;
					render();
				}
			} else {
				if (rendered === true) {
					rendered = false;
					el.innerHTML = '';
				}
			}
		});

		observer.observe(el);
		return () => observer.unobserve(el);
	});

	const render = async () => {
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
			.attr('viewBox', [0, 0, $rightWidth, height])
			.attr('width', $rightWidth)
			.attr('height', height);

		let xScale = d3
			.scaleTime()
			.domain(domain)
			.range([margin.left, $rightWidth - margin.right]);

		let xAxis = d3.axisTop(xScale).tickSize(-height).tickSizeOuter(0);

		let g = svg.append('g').attr('transform', 'translate(0, 0)');
		g.call(xAxis);
		g.selectAll('text').remove();
		g.selectAll('.domain').remove();
		g.selectAll('line').attr('stroke', '#aaa');

		const color = d3
			.scaleOrdinal()
			.domain(keys)
			.range(['#FF423D', '#4CF6C3', '#0068FF', 'transparent']);

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

		svg
			.append('text')
			.style('font-size', '10px')
			.attr('transform', `translate(5, 15)`)
			.attr('fill', 'white')
			.text(my_information.name);

		const tooltip = svg.append('g').attr('opacity', 0);
		tooltip.append('rect').attr('width', 1).attr('height', height).attr('fill', '#fff');
		const textColor = '#fff';
		const rectColor = '#000';
		const tooltipWidth = 80;
		const tooltipInfo = tooltip.append('g');
		tooltipInfo
			.append('rect')
			.attr('width', tooltipWidth)
			.attr('height', 50)
			.attr('fill', rectColor)
			.attr('transform', 'translate(1, 12)');
		tooltipInfo
			.append('text')
			.attr('class', 'data-bikes')
			.style('font-size', '10px')
			.attr('transform', `translate(5, 25)`)
			.attr('fill', textColor);
		tooltipInfo
			.append('text')
			.attr('class', 'data-ebikes')
			.style('font-size', '10px')
			.attr('transform', `translate(5, 35)`)
			.attr('fill', textColor);
		tooltipInfo
			.append('text')
			.attr('class', 'data-disabled')
			.style('font-size', '10px')
			.attr('transform', `translate(5, 45)`)
			.attr('fill', textColor);
		tooltipInfo
			.append('text')
			.attr('class', 'data-date')
			.style('font-size', '10px')
			.attr('transform', `translate(5, 55)`)
			.attr('fill', textColor);

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
				tooltip.attr('opacity', 1);
				let [x, date] = ruleValue;
				const d = data.find((row) => row.times > date);
				if (!d) return;
				if (x > $rightWidth - tooltipWidth) {
					tooltipInfo.attr('transform', `translate(${-tooltipWidth - 1}, 0)`);
				} else {
					tooltipInfo.attr('transform', `translate(0, 0)`);
				}
				tooltip.attr('transform', `translate(${x}, 0)`);
				tooltip.select('text.data-bikes').text(`${d.num_bikes_available} bikes`);
				tooltip.select('text.data-ebikes').text(`${d.num_ebikes_available} ebikes`);
				tooltip.select('text.data-disabled').text(`${d.num_bikes_disabled} disabled`);
				tooltip.select('text.data-date').text(`${date.toLocaleTimeString()}`);
			}
		});
		return svg;
	};
</script>

<!-- markup (zero or more items) goes here -->
<div class="row" id="station-{my_id}">
	<div bind:this={el} class="chart" />
</div>

<style>
	.row {
		line-height: 1;
		height: 100px;
		display: grid;
		grid-template-columns: 150px 1fr;
		border-bottom: 1px solid white;
	}
	.chart {
		background: #111;
		line-height: 0;
	}
</style>
