<script>
    let name = "world";
</script>

<h1>Welcome to SvelteKit</h1>
<input bind:value={name} placeholder="Enter your name" />
{#if name}
    <h2>Hello {name}!</h2>
{:else}
    <h2>Hello, Guest!</h2>
{/if}
