<template>
	<div class="route-skeleton" aria-hidden="true">
		<div class="skeleton-heading skeleton-shimmer"></div>
		<div class="skeleton-toolbar">
			<div class="skeleton-search skeleton-shimmer"></div>
			<div class="skeleton-action skeleton-shimmer"></div>
		</div>
		<div class="skeleton-filters">
			<div class="skeleton-chip skeleton-shimmer"></div>
			<div class="skeleton-chip skeleton-chip-short skeleton-shimmer"></div>
		</div>
		<div class="skeleton-grid">
			<div v-for="item in 6" :key="item" class="skeleton-card">
				<div class="skeleton-icon skeleton-shimmer"></div>
				<div class="skeleton-copy">
					<div class="skeleton-line skeleton-line-title skeleton-shimmer"></div>
					<div class="skeleton-line skeleton-shimmer"></div>
					<div class="skeleton-line skeleton-line-short skeleton-shimmer"></div>
				</div>
			</div>
		</div>
	</div>
</template>

<style scoped>
.route-skeleton {
	position: absolute;
	inset: 0;
	z-index: 20;
	padding: 1.5rem;
	overflow: hidden;
	pointer-events: none;
	background: var(--color-bg);
}

.skeleton-heading {
	width: min(13rem, 38%);
	height: 1.75rem;
	margin-bottom: 1rem;
	border-radius: var(--radius-md);
}

.skeleton-toolbar,
.skeleton-filters,
.skeleton-card {
	display: flex;
}

.skeleton-toolbar {
	gap: 0.75rem;
}

.skeleton-search {
	flex: 1;
	height: 2.25rem;
	border-radius: var(--radius-md);
}

.skeleton-action {
	width: 9.5rem;
	height: 2.25rem;
	border-radius: var(--radius-md);
}

.skeleton-filters {
	gap: 0.5rem;
	margin-top: 0.75rem;
}

.skeleton-chip {
	width: 8.5rem;
	height: 2rem;
	border-radius: var(--radius-max);
}

.skeleton-chip-short {
	width: 6.5rem;
}

.skeleton-grid {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr));
	gap: 0.75rem;
	margin-top: 1.25rem;
}

.skeleton-card {
	align-items: center;
	gap: 0.875rem;
	min-height: 6.5rem;
	padding: 1rem;
	border: 1px solid var(--surface-4);
	border-radius: var(--radius-lg);
	background: color-mix(in srgb, var(--surface-3) 72%, transparent);
}

.skeleton-icon {
	width: 4.25rem;
	height: 4.25rem;
	flex-shrink: 0;
	border-radius: var(--radius-md);
}

.skeleton-copy {
	display: flex;
	flex: 1;
	flex-direction: column;
	gap: 0.55rem;
}

.skeleton-line {
	width: 88%;
	height: 0.65rem;
	border-radius: var(--radius-sm);
}

.skeleton-line-title {
	width: 62%;
	height: 0.9rem;
}

.skeleton-line-short {
	width: 45%;
}

.skeleton-shimmer {
	background: linear-gradient(
		100deg,
		var(--surface-4) 20%,
		color-mix(in srgb, var(--surface-5) 68%, var(--surface-4)) 42%,
		var(--surface-4) 64%
	);
	background-size: 240% 100%;
	animation: skeleton-shimmer 1.65s ease-in-out infinite;
}

@keyframes skeleton-shimmer {
	from {
		background-position: 110% 0;
	}

	to {
		background-position: -110% 0;
	}
}

@media (prefers-reduced-motion: reduce) {
	.skeleton-shimmer {
		animation: none;
	}
}
</style>
