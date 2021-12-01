<script lang="ts">
	import { browser } from '$app/env';
	import { goto } from '$app/navigation';
	import { jobId } from '$lib/stores';
	import { Button, Card, CardBody, CardHeader, Icon } from 'sveltestrap';

	type ServerError = {
		kind: string;
		message: string;
	};

	type StatusResponse = {
		status:
			| 'Submitted'
			| 'CheckingToken'
			| 'BuildingScript'
			| 'Uploading'
			| 'CreatingCronJon'
			| { Error: ServerError }
			| { Done: string };
	};

	enum JobStatus {
		Submitted = 0,
		CheckingToken,
		BuildingScript,
		Uploading,
		CreatingCronJob,
		Done,
		Error = -1
	}

	let status: JobStatus = JobStatus.Error;
	let done = '';
	let error: ServerError;

	const pollServer = async () => {
		try {
			console.log('Running request');
			let request = await fetch(import.meta.env.VITE_API_URL + 'status/' + $jobId, {
				method: 'GET'
			});
			console.log('Request resolved');
			let data: StatusResponse = await request.json();

			console.log(data);

			if (typeof data.status == 'string') {
				switch (data.status) {
					case 'Submitted':
						status = JobStatus.Submitted;
						break;
					case 'CheckingToken':
						status = JobStatus.CheckingToken;
						break;
					case 'BuildingScript':
						status = JobStatus.BuildingScript;
						break;
					case 'Uploading':
						status = JobStatus.Uploading;
						break;
					case 'CreatingCronJon':
						status = JobStatus.CreatingCronJob;
						break;
				}
			} else if ('Done' in data.status) {
				status = JobStatus.Done;
				done = data.status.Done;
				jobId.set(null);
				return;
			} else {
				status = JobStatus.Error;
				error = data.status.Error;
				jobId.set(null);
				return;
			}
		} catch (ex) {
			console.log(ex);
			jobId.set(null);
			status = JobStatus.Error;
			error = {
				kind: 'Server error',
				message: 'Server responeded with error' + ex.response.status
			};
			return;
		}

		setTimeout(pollServer, 200);
	};

	let initalTimeout = setTimeout(pollServer, 100);

	if ($jobId == null && browser) {
		clearTimeout(initalTimeout);
		goto('/');
	}
</script>

<div class="top">
	<h1>Running Job</h1>
	<p>Please be paitent while you job is processed</p>
</div>
{#key status}
	<div class="cards">
		<Card
			class={'cards__card ' + (status == JobStatus.CheckingToken ? 'cards__card--active' : '')}
			color={status > JobStatus.CheckingToken
				? 'success'
				: status == JobStatus.CheckingToken
				? 'light'
				: 'secondary'}
		>
			<CardHeader
				><Icon name="list-check" class="cards__card__icon" /><br />Check Cloudfload Token</CardHeader
			>
			<CardBody>
				{#if status == JobStatus.CheckingToken}
					<Icon name="arrow-clockwise" class="spin" /><br />
					Running
				{:else if status > JobStatus.CheckingToken}
					<Icon name="check-circle" /><br />
					Done
				{:else}
					Waiting
				{/if}
			</CardBody>
		</Card>
		<Card
			class={'cards__card ' + (status == JobStatus.BuildingScript ? 'cards__card--active' : '')}
			color={status > JobStatus.BuildingScript
				? 'success'
				: status == JobStatus.BuildingScript
				? 'light'
				: 'secondary'}
		>
			<CardHeader
				><Icon name="code-square" class="cards__card__icon" /><br />Compile Script</CardHeader
			>
			<CardBody>
				{#if status == JobStatus.BuildingScript}
					<Icon name="arrow-clockwise" class="spin" /><br />
					Running
				{:else if status > JobStatus.BuildingScript}
					<Icon name="check-circle" /><br />
					Done
				{:else}
					Waiting
				{/if}
			</CardBody>
		</Card>
		<Card
			class={'cards__card ' + (status == JobStatus.Uploading ? 'cards__card--active' : '')}
			color={status > JobStatus.Uploading
				? 'success'
				: status == JobStatus.Uploading
				? 'light'
				: 'secondary'}
		>
			<CardHeader
				><Icon name="cloud-upload" class="cards__card__icon" /><br />Upload to your server</CardHeader
			>
			<CardBody>
				{#if status == JobStatus.Uploading}
					<Icon name="arrow-clockwise" class="spin" /><br />
					Running
				{:else if status > JobStatus.Uploading}
					<Icon name="check-circle" /><br />
					Done
				{:else}
					Waiting
				{/if}
			</CardBody>
		</Card>
		<Card
			class={'cards__card ' + (status == JobStatus.CreatingCronJob ? 'cards__card--active' : '')}
			color={status > JobStatus.CreatingCronJob
				? 'success'
				: status == JobStatus.CreatingCronJob
				? 'light'
				: 'secondary'}
		>
			<CardHeader
				><Icon name="terminal" class="cards__card__icon" /><br />Create cron job</CardHeader
			>
			<CardBody>
				{#if status == JobStatus.CreatingCronJob}
					<Icon name="arrow-clockwise" class="spin" /><br />
					Running
				{:else if status > JobStatus.CreatingCronJob}
					<Icon name="check-circle" /><br />
					Done
				{:else}
					Waiting
				{/if}
			</CardBody>
		</Card>
	</div>
{/key}
{#if status == JobStatus.Done}
	<div class="done-card--wrapper">
		<Card color="success" class="done-card">
			<CardHeader><h1><Icon name="check-circle" /> Done</h1></CardHeader>
			<CardBody><h5>Successfull uploaded {done} and created cronjob to run it</h5></CardBody>
		</Card>
		<Button
			on:click={() => {
				goto('/');
			}}>Back Home</Button
		>
	</div>
{/if}
{#if status == JobStatus.Error}
	<div class="done-card--wrapper">
		<Card color="danger" class="done-card">
			<CardHeader><h1><Icon name="x-circle" /> Error</h1></CardHeader>
			<CardBody>
				{#if typeof error != 'undefined' && 'kind' in error}
					<h6>{error.kind}</h6>
					<p>{error.message}</p>
				{/if}
			</CardBody>
		</Card>
		<Button
			on:click={() => {
				goto('/');
			}}>Back Home</Button
		>
	</div>
{/if}

<style>
	.top {
		text-align: center;
		padding: 2rem;
		background-color: black;
		color: white;
	}

	.top h1 {
		padding: 1rem;
	}
	.cards {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
		flex-grow: 2;
		align-items: stretch;
		justify-content: center;
		padding: 1rem;
	}

	:global(.cards__card) {
		max-width: 20%;
		text-align: center;
	}

	:global(.cards__card__icon) {
		font-size: 3rem;
	}

	:global(.spin::before) {
		animation: spin 1s infinite linear;
	}

	.done-card--wrapper {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		align-items: center;
		justify-content: center;
		padding: 2rem;
	}

	:global(.done-card) {
		width: fit-content;
		text-align: center;
	}

	@keyframes spin {
		0% {
			transform: rotate(0deg);
		}
		50% {
			transform: rotate(180deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}
</style>
