<script>
	import { onMount } from 'svelte';
	import * as d3 from 'd3';
	export let domain;
	let el;

	let margin = { top: 0, right: 0, bottom: 0, left: 0 };
	let height = 20;
	let width = 600;
	let x = d3
		.scaleTime()
		.domain(domain)
		.range([margin.left, width - margin.right]);

	let xAxis = d3.axisTop(x).tickSize(2).tickSizeOuter(0);
	onMount(() => {
		let svg = d3.select(el).append('svg').attr('viewBox', [0, 0, width, height]);
		let g = svg.append('g').attr('transform', 'translate(0, 19)');
		g.call(xAxis);
	});
</script>

<div class="axis" bind:this={el} />

<style>
	.axis {
		color: white;
	}
	:global(.axis svg g text) {
		font-size: 4px;
	}
</style>
