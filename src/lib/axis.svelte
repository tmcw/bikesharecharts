<script lang="ts">
	import { onMount } from 'svelte';
	import * as d3 from 'd3';
	import Suncalc from 'suncalc';
	import { rightWidth } from './stores';
	export let domain;
	let el: HTMLDivElement;

	let margin = { top: 0, right: 0, bottom: 0, left: 0 };
	let height = 40;
	let x = d3
		.scaleTime()
		.domain(domain)
		.range([margin.left, $rightWidth - margin.right]);
	const days = d3.timeDay.range(domain[0], d3.timeDay.offset(domain[1], 1));

	const sunTimes = days.map((d) => {
		return Suncalc.getTimes(d, 40.7128, -74.006);
	});

	console.log(sunTimes);

	let xAxis = d3.axisTop(x).tickSize(2).tickSizeOuter(0);

	onMount(() => {
		let svg = d3
			.select(el)
			.append('svg')
			.attr('viewBox', [0, 0, $rightWidth, height])
			.attr('width', $rightWidth)
			.attr('height', height);
		let g = svg.append('g').attr('transform', 'translate(0, 19)');
		g.call(xAxis);
		g.selectAll('path').remove();
		let sunHeight = 10;
		let sun = svg.append('g').attr('transform', 'translate(0, 19)');
		sun
			.selectAll('rect')
			.data(sunTimes)
			.join('rect')
			.attr('x', (d) => x(d.sunrise))
			.attr('y', 0)
			.attr('width', (d) => x(d.sunset) - x(d.sunrise))
			.attr('height', sunHeight / 2)
			.attr('fill', 'yellow');
	});
</script>

<div class="axis" bind:this={el} />

<style>
	.axis {
		padding-right: 20px;
		color: white;
	}
	:global(.axis svg g text) {
		font-size: 10px;
		font-family: 'IBM Plex Mono', monospace;
	}
</style>
