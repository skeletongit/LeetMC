<script setup lang="ts">
import {
	BoxesIcon,
	ChevronRightIcon,
	ClockIcon,
	LibraryIcon,
	PlayIcon,
	PlusIcon,
	SpinnerIcon,
	StopCircleIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	defineMessages,
	injectNotificationManager,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import dayjs from 'dayjs'
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { getInstanceIconUrl, kill, run } from '@/helpers/instance'
import { get_by_instance_id } from '@/helpers/process'
import type { GameInstance } from '@/helpers/types'

import LibrarySection from './index.vue'

const props = defineProps<{
	instances: GameInstance[]
}>()

const emit = defineEmits<{
	create: []
}>()

const messages = defineMessages({
	title: {
		id: 'app.modern-interface.home.title',
		defaultMessage: 'Ready when you are',
	},
	description: {
		id: 'app.modern-interface.home.description',
		defaultMessage: 'Jump back into your latest instance or shape the next one.',
	},
	continuePlaying: {
		id: 'app.modern-interface.home.continue-playing',
		defaultMessage: 'Continue playing',
	},
	play: {
		id: 'app.instance.card.play',
		defaultMessage: 'Play',
	},
	stop: {
		id: 'app.instance.card.stop',
		defaultMessage: 'Stop',
	},
	openInstance: {
		id: 'app.modern-interface.home.open-instance',
		defaultMessage: 'Open instance',
	},
	createInstance: {
		id: 'app.nav.create-new-instance',
		defaultMessage: 'Create new instance',
	},
	recentTitle: {
		id: 'app.modern-interface.home.recent-title',
		defaultMessage: 'Recently played',
	},
	allInstances: {
		id: 'app.modern-interface.home.instance-count',
		defaultMessage: '{count, plural, one {# instance} other {# instances}}',
	},
	played: {
		id: 'app.instance.card.played',
		defaultMessage: 'Played {relativeTime}',
	},
	neverPlayed: {
		id: 'app.instance.card.never-played',
		defaultMessage: 'Never played',
	},
	emptyTitle: {
		id: 'app.modern-interface.home.empty-title',
		defaultMessage: 'Your game space is ready',
	},
	emptyDescription: {
		id: 'app.modern-interface.home.empty-description',
		defaultMessage: 'Create your first instance and LeetMC will keep everything organised here.',
	},
})

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const { handleError } = injectNotificationManager()
const router = useRouter()
const hero = computed(() => props.instances[0] ?? null)
const recentInstances = computed(() => props.instances.slice(1, 5))
const playing = ref(false)
const loading = ref(false)

function instanceIcon(instance: GameInstance): string | null {
	return getInstanceIconUrl(instance.icon_path)
}

function lastPlayed(instance: GameInstance): string {
	if (!instance.last_played) return formatMessage(messages.neverPlayed)
	return formatMessage(messages.played, {
		relativeTime: formatRelativeTime(dayjs(instance.last_played).toISOString()),
	})
}

async function refreshPlayingState(): Promise<void> {
	if (!hero.value) {
		playing.value = false
		return
	}
	const processes = await get_by_instance_id(hero.value.id).catch(() => [])
	playing.value = processes.length > 0
}

async function togglePlay(): Promise<void> {
	if (!hero.value || loading.value) return
	loading.value = true
	try {
		if (playing.value) {
			await kill(hero.value.id)
			playing.value = false
		} else {
			await run(hero.value.id)
		}
	} catch (error) {
		if (playing.value) handleError(error)
		else handleSevereError(error, { instanceId: hero.value.id })
	} finally {
		loading.value = false
	}
}

async function openInstance(instance: GameInstance): Promise<void> {
	await router.push(`/instance/${encodeURIComponent(instance.id)}`)
}

watch(() => hero.value?.id, refreshPlayingState, { immediate: true })

useAppEvent('process', (event) => {
	if (event.instance_id !== hero.value?.id) return
	if (event.event === 'launched') playing.value = true
	if (event.event === 'finished') playing.value = false
})
</script>

<template>
	<div class="modern-home" data-library-page-background>
		<header class="modern-home-heading">
			<div class="min-w-0">
				<h1>{{ formatMessage(messages.title) }}</h1>
				<p>{{ formatMessage(messages.description) }}</p>
			</div>
			<Button type="outlined" class="shrink-0" @click="emit('create')">
				<PlusIcon />
				{{ formatMessage(messages.createInstance) }}
			</Button>
		</header>

		<section v-if="hero" class="modern-hero liquid-glass-panel">
			<img class="modern-hero-backdrop" :src="instanceIcon(hero)" alt="" aria-hidden="true" />
			<div class="modern-hero-wash"></div>
			<div class="modern-hero-content">
				<div class="modern-hero-copy">
					<span class="modern-kicker">{{ formatMessage(messages.continuePlaying) }}</span>
					<h2>{{ hero.name }}</h2>
					<div class="modern-meta">
						<span>{{ hero.loader }} {{ hero.game_version }}</span>
						<span><ClockIcon /> {{ lastPlayed(hero) }}</span>
					</div>
					<div class="modern-hero-actions">
						<Button
							v-if="hero.install_stage === 'installed' && !hero.quarantined"
							type="colored"
							:color="playing ? 'red' : 'brand'"
							:disabled="loading"
							@click="togglePlay"
						>
							<SpinnerIcon v-if="loading" class="animate-spin" />
							<StopCircleIcon v-else-if="playing" />
							<PlayIcon v-else />
							{{ formatMessage(playing ? messages.stop : messages.play) }}
						</Button>
						<Button type="outlined" @click="openInstance(hero)">
							{{ formatMessage(messages.openInstance) }}
							<ChevronRightIcon />
						</Button>
					</div>
				</div>
				<Avatar
					class="modern-hero-icon"
					size="clamp(8rem, 20vw, 14rem)"
					:src="instanceIcon(hero)"
					:tint-by="hero.id"
					alt=""
					no-shadow
					pad-transparent-corners
				/>
			</div>
		</section>

		<section v-else class="modern-empty liquid-glass-panel">
			<div class="modern-empty-icon"><LibraryIcon /></div>
			<h2>{{ formatMessage(messages.emptyTitle) }}</h2>
			<p>{{ formatMessage(messages.emptyDescription) }}</p>
			<Button type="colored" color="brand" @click="emit('create')">
				<PlusIcon />
				{{ formatMessage(messages.createInstance) }}
			</Button>
		</section>

		<section v-if="recentInstances.length" class="modern-recent">
			<div class="modern-section-heading">
				<div>
					<h2>{{ formatMessage(messages.recentTitle) }}</h2>
					<p>{{ formatMessage(messages.allInstances, { count: instances.length }) }}</p>
				</div>
				<BoxesIcon />
			</div>
			<div class="modern-recent-grid">
				<button
					v-for="instance in recentInstances"
					:key="instance.id"
					type="button"
					class="modern-recent-card liquid-glass-panel"
					@click="openInstance(instance)"
				>
					<Avatar
						size="3.5rem"
						:src="instanceIcon(instance)"
						:tint-by="instance.id"
						alt=""
						no-shadow
						pad-transparent-corners
					/>
					<span class="min-w-0 flex-1 text-left">
						<strong>{{ instance.name }}</strong>
						<small>{{ instance.loader }} {{ instance.game_version }}</small>
					</span>
					<ChevronRightIcon class="shrink-0 text-secondary" />
				</button>
			</div>
		</section>

		<div v-if="instances.length" class="modern-library">
			<LibrarySection :instances="instances" />
		</div>
	</div>
</template>

<style scoped>
.modern-home {
	display: flex;
	min-height: 100%;
	flex-direction: column;
	gap: clamp(1.25rem, 2.5vw, 2rem);
	padding: clamp(1rem, 2.5vw, 2rem);
}

.modern-home-heading {
	display: flex;
	align-items: flex-end;
	justify-content: space-between;
	gap: 1.5rem;
}

.modern-home-heading h1 {
	margin: 0.25rem 0 0;
	color: var(--color-contrast);
	font-size: clamp(1.75rem, 4vw, 3rem);
	font-weight: 750;
	letter-spacing: -0.045em;
	line-height: 1.05;
}

.modern-home-heading p {
	margin: 0.65rem 0 0;
	color: var(--color-secondary);
	font-size: clamp(0.9rem, 1.4vw, 1.05rem);
}

.modern-meta span,
.modern-section-heading {
	display: flex;
	align-items: center;
}

.modern-meta svg {
	width: 1rem;
	height: 1rem;
}

.modern-hero {
	position: relative;
	min-height: clamp(18rem, 37vw, 27rem);
	overflow: hidden;
	border: 1px solid var(--liquid-glass-border);
	border-radius: clamp(1.5rem, 3vw, 2.25rem);
	background:
		radial-gradient(circle at 78% 45%, color-mix(in srgb, var(--color-brand) 22%, transparent), transparent 38%),
		linear-gradient(115deg, color-mix(in srgb, var(--color-bg-raised) 92%, transparent), color-mix(in srgb, var(--color-bg) 55%, transparent));
	box-shadow: 0 1.5rem 4rem color-mix(in srgb, #000 24%, transparent);
}

.modern-hero-backdrop {
	position: absolute;
	top: -35%;
	right: -5%;
	width: 55%;
	height: 170%;
	object-fit: contain;
	filter: blur(3.5rem) saturate(1.35);
	opacity: 0.22;
	transform: scale(1.25);
}

.modern-hero-wash {
	position: absolute;
	inset: 0;
	background:
		linear-gradient(90deg, color-mix(in srgb, var(--color-bg-raised) 95%, transparent) 0%, transparent 72%),
		linear-gradient(0deg, color-mix(in srgb, var(--color-bg) 42%, transparent), transparent 55%);
}

.modern-hero-content {
	position: relative;
	z-index: 1;
	display: grid;
	min-height: inherit;
	grid-template-columns: minmax(0, 1fr) auto;
	align-items: center;
	gap: clamp(1rem, 4vw, 4rem);
	padding: clamp(1.5rem, 5vw, 4rem);
}

.modern-hero-copy {
	min-width: 0;
}

.modern-kicker {
	display: inline-flex;
	border: 1px solid color-mix(in srgb, var(--color-brand) 42%, transparent);
	border-radius: 999px;
	background: var(--color-brand-highlight);
	padding: 0.35rem 0.7rem;
	color: var(--color-brand);
	font-size: 0.75rem;
	font-weight: 700;
}

.modern-hero h2 {
	margin: 0.85rem 0 0;
	color: var(--color-contrast);
	font-size: clamp(2rem, 5vw, 4.25rem);
	font-weight: 800;
	letter-spacing: -0.055em;
	line-height: 0.98;
	text-wrap: balance;
}

.modern-meta {
	display: flex;
	flex-wrap: wrap;
	gap: 0.65rem 1rem;
	margin-top: 1rem;
	color: var(--color-secondary);
	font-size: 0.875rem;
	font-weight: 600;
	text-transform: capitalize;
}

.modern-meta span {
	gap: 0.35rem;
}

.modern-hero-actions {
	display: flex;
	flex-wrap: wrap;
	gap: 0.65rem;
	margin-top: 1.5rem;
}

.modern-hero-icon {
	filter: drop-shadow(0 1.75rem 2rem color-mix(in srgb, #000 35%, transparent));
	transition: transform 300ms cubic-bezier(0.2, 0.8, 0.2, 1);
}

.modern-hero:hover .modern-hero-icon {
	transform: translateY(-0.35rem) scale(1.025);
}

.modern-section-heading {
	justify-content: space-between;
	gap: 1rem;
	margin-bottom: 0.85rem;
}

.modern-section-heading h2,
.modern-empty h2 {
	margin: 0;
	color: var(--color-contrast);
	font-size: 1.25rem;
}

.modern-section-heading p,
.modern-empty p {
	margin: 0.25rem 0 0;
	color: var(--color-secondary);
	font-size: 0.875rem;
}

.modern-section-heading > svg {
	width: 1.35rem;
	height: 1.35rem;
	color: var(--color-brand);
}

.modern-recent-grid {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 0.75rem;
}

.modern-recent-card {
	display: flex;
	min-width: 0;
	align-items: center;
	gap: 0.85rem;
	border: 1px solid var(--liquid-glass-border);
	border-radius: 1.25rem;
	background: var(--liquid-glass-bg);
	padding: 0.8rem;
	color: var(--color-primary);
	cursor: pointer;
	transition: transform 160ms ease, border-color 160ms ease, background-color 160ms ease;
}

.modern-recent-card:hover {
	transform: translateY(-2px);
	border-color: color-mix(in srgb, var(--color-brand) 46%, var(--liquid-glass-border));
	background: color-mix(in srgb, var(--liquid-glass-bg) 85%, var(--color-brand-highlight));
}

.modern-recent-card strong,
.modern-recent-card small {
	display: block;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.modern-recent-card strong {
	color: var(--color-contrast);
	font-size: 0.95rem;
}

.modern-recent-card small {
	margin-top: 0.25rem;
	color: var(--color-secondary);
	font-size: 0.75rem;
	text-transform: capitalize;
}

.modern-empty {
	display: flex;
	min-height: 24rem;
	align-items: center;
	justify-content: center;
	flex-direction: column;
	border: 1px solid var(--liquid-glass-border);
	border-radius: 2rem;
	padding: 3rem 1.5rem;
	text-align: center;
}

.modern-empty p {
	max-width: 32rem;
	margin-bottom: 1.25rem;
}

.modern-empty-icon {
	display: grid;
	width: 4rem;
	height: 4rem;
	place-items: center;
	margin-bottom: 1rem;
	border-radius: 1.25rem;
	background: var(--color-brand-highlight);
	color: var(--color-brand);
}

.modern-empty-icon svg {
	width: 2rem;
	height: 2rem;
}

.modern-library {
	padding-top: 0.5rem;
}

:global(html.light-mode) .modern-hero {
	background:
		radial-gradient(circle at 80% 40%, color-mix(in srgb, var(--color-brand) 18%, white), transparent 42%),
		linear-gradient(120deg, color-mix(in srgb, white 82%, var(--color-brand-highlight)), color-mix(in srgb, var(--color-bg) 76%, white));
	box-shadow: 0 1.5rem 4rem rgb(91 61 69 / 13%);
}

:global(html.oled-mode) .modern-hero {
	border-color: color-mix(in srgb, var(--color-brand) 24%, var(--liquid-glass-border));
	background:
		radial-gradient(circle at 78% 45%, color-mix(in srgb, var(--color-brand) 18%, transparent), transparent 36%),
		#020504;
	box-shadow: 0 1.5rem 4rem rgb(0 0 0 / 52%);
}

:global(html.retro-mode) .modern-hero {
	border-radius: 1.15rem;
	background:
		linear-gradient(90deg, color-mix(in srgb, var(--color-brand) 5%, transparent) 1px, transparent 1px),
		linear-gradient(color-mix(in srgb, var(--color-brand) 5%, transparent) 1px, transparent 1px),
		linear-gradient(120deg, var(--color-bg-raised), var(--color-bg));
	background-size: 16px 16px, 16px 16px, auto;
	box-shadow: 0.55rem 0.55rem 0 color-mix(in srgb, var(--color-brand) 12%, transparent);
}

:global(html.retro-mode) .modern-recent-card,
:global(html.retro-mode) .modern-empty {
	border-radius: 0.85rem;
}

@media (max-width: 760px) {
	.modern-home-heading {
		align-items: stretch;
		flex-direction: column;
	}

	.modern-home-heading :deep(button) {
		align-self: flex-start;
	}

	.modern-hero-content {
		grid-template-columns: minmax(0, 1fr);
	}

	.modern-hero-icon {
		position: absolute;
		right: -2rem;
		bottom: -2.5rem;
		opacity: 0.22;
	}

	.modern-hero-copy {
		position: relative;
		z-index: 1;
	}

	.modern-recent-grid {
		grid-template-columns: minmax(0, 1fr);
	}
}

@media (prefers-reduced-motion: reduce) {
	.modern-hero-icon,
	.modern-recent-card {
		transition: none;
	}
}
</style>
