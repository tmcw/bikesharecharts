<script lang="ts">
	import { onMount } from 'svelte';
	import StationRow from '$lib/station_row.svelte';

	export let context: duckdb.AsyncDuckDBConnection;
	export let id: string;
	export let domain: [Date, Date];
	export let id_legend;
	export let station_information;

	let el: HTMLDivElement;

	let intersecting = false;

	onMount(() => {
		const observer = new IntersectionObserver((entries) => {
			intersecting = entries[0].isIntersecting;
		});

		observer.observe(el);
		return () => observer.unobserve(el);
	});
</script>

<!-- markup (zero or more items) goes here -->
<div class="row" bind:this={el}>
	{#if intersecting}
		<StationRow {...$$props} />
	{/if}
</div>

<style>
	.row {
		line-height: 1;
		height: 100px;
		display: grid;
		grid-template-columns: 150px 1fr;
		border-bottom: 1px solid white;
	}
</style>
