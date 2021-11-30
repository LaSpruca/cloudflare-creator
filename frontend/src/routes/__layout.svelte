<script lang="ts">
	import { Styles } from 'sveltestrap';
</script>

<Styles />

<div class="main">
	<slot />
</div>

<style>
	:global(body) {
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: grey;
		min-height: 100vh;
	}

	.main {
		background-color: white;
		height: max-content;
		overflow: hidden;
		border-radius: 5px;
		margin: 5rem 3rem;
	}
</style>
