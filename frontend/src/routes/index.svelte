<script lang="ts">
	import MainForm, { AuthMethod } from '$lib/formData';
	import { get, Writable, writable } from 'svelte/store';
	import { Button, FormGroup, Input, InputGroup, InputGroupText } from 'sveltestrap';

	const setupUpdate = () => {
		console.log('Uploading Script');
		console.log(JSON.stringify(get(formData)));
	};

	const formData: Writable<MainForm> = writable(new MainForm());
</script>

<div class="top">
	<h1>Cloudflare Auto DNS Update</h1>
	<p>
		Generate an script to automatically update a cloudflare dns record and upload it to a server
		over SSH
	</p>
</div>

<div class="input-list">
	<InputGroup class="input-list__input">
		<InputGroupText class="input-list__input__label">Cloudflare Token</InputGroupText>
		<Input
			placeholder="Cloudflare Token"
			bind:value={$formData.cfToken.value}
			valid={$formData.cfToken.isValid()}
		/>
	</InputGroup>
	<InputGroup class="input-list__input">
		<InputGroupText class="input-list__input__label">Cloudflare Email</InputGroupText>
		<Input
			placeholder="Cloudflare Email"
			bind:value={$formData.cfEmail.value}
			type="email"
			valid={$formData.cfEmail.isValid()}
		/>
	</InputGroup>
	<InputGroup class="input-list__input">
		<InputGroupText class="input-list__input__label">Cloudflare Zone</InputGroupText>
		<Input
			placeholder="Cloudflare Zone"
			bind:value={$formData.cfZone.value}
			valid={$formData.cfZone.isValid()}
		/>
	</InputGroup>
	<InputGroup class="input-list__input">
		<InputGroupText class="input-list__input__label">DNS Record</InputGroupText>
		<Input
			placeholder="DNS Record"
			bind:value={$formData.cfDns.value}
			valid={$formData.cfDns.isValid()}
		/>
	</InputGroup>
</div>

<div class="input-list">
	<InputGroup class="input-list__input">
		<InputGroupText class="input-list__input__label">Server address</InputGroupText>
		<Input
			placeholder="Server address"
			bind:value={$formData.sshAddress.value}
			valid={$formData.sshAddress.isValid()}
		/>
	</InputGroup>
	<InputGroup class="input-list__input">
		<InputGroupText class="input-list__input__label">Port</InputGroupText>
		<Input
			placeholder="22"
			bind:value={$formData.sshPort.value}
			valid={$formData.sshPort.isValid()}
		/>
	</InputGroup>
	<InputGroup class="input-list__input">
		<InputGroupText class="input-list__input__label">Username</InputGroupText>
		<Input
			placeholder="root"
			bind:value={$formData.sshUsername.value}
			valid={$formData.sshUsername.isValid()}
		/>
	</InputGroup>
	<FormGroup inline={true}>
		<InputGroupText class="input-list__input__label" style="width: min-content"
			>Authentication Method</InputGroupText
		>
		<Input
			type="radio"
			label="Password"
			bind:group={$formData.sshAuthMethod}
			value={AuthMethod.Password}
		/>
		<Input
			type="radio"
			label="RSA Key"
			bind:group={$formData.sshAuthMethod}
			value={AuthMethod.Key}
		/>
	</FormGroup>
	{#if $formData.sshAuthMethod == AuthMethod.Password}
		<InputGroup class="input-list__input">
			<InputGroupText class="input-list__input__label">Password</InputGroupText>
			<Input
				placeholder="ThisIsNotYourPassword"
				bind:value={$formData.sshPassword.value}
				valid={$formData.sshPassword.isValid()}
			/>
		</InputGroup>
	{:else}
		<InputGroup class="input-list__input">
			<InputGroupText class="input-list__input__label">RSA Key</InputGroupText>
			<Input
				placeholder="Your RSA Key here"
				type="textarea"
				bind:value={$formData.sshRsaKey.value}
				valid={$formData.sshRsaKey.isValid()}
			/>
		</InputGroup>
	{/if}
</div>

<div class="submit--wrapper">
	<Button class="submit" disabled={!$formData.isValid()} on:click={setupUpdate}>Setup update</Button
	>
</div>

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

	.input-list {
		padding: 1rem 0;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		gap: 1rem;
	}

	:global(.input-list__input) {
		width: 80%;
	}

	:global(.input-list__input__label) {
		width: 10rem;
	}

	.submit--wrapper {
		padding: 1rem;
		display: flex;
		justify-content: center;
		align-items: center;
	}
</style>
