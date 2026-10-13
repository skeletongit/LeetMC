<script setup lang="ts">
import { ImportIcon, LanguagesIcon, ModrinthIcon, PlusIcon } from '@modrinth/assets'
import {
	Button,
	type ButtonMenuOption,
	defineMessages,
	injectI18n,
	LOCALES,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { computed, inject, onMounted, onUnmounted, ref } from 'vue'

import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'

const showCreationModal = inject<() => void>('showCreationModal')
const showImportModal = inject<() => void>('showImportModal')

const { formatMessage } = useVIntl()
const { locale, setLocale } = injectI18n()

const messages = defineMessages({
	welcomeTitle: {
		id: 'app.welcome-screen.title',
		defaultMessage: 'Welcome to LeetMC',
	},
	welcomeDescription: {
		id: 'app.welcome-screen.description',
		defaultMessage: 'Created by players, for players',
	},
	createInstance: {
		id: 'app.welcome-screen.create-instance',
		defaultMessage: 'Create an instance',
	},
	importPrompt: {
		id: 'app.welcome-screen.import-prompt',
		defaultMessage: 'Escaping another launcher?',
	},
	importFromLauncher: {
		id: 'app.welcome-screen.import-from-launcher',
		defaultMessage: 'Import from launcher',
	},
	language: {
		id: 'app.welcome-screen.language',
		defaultMessage: 'Language',
	},
})

const languageOptions = computed<ButtonMenuOption[]>(() =>
	LOCALES.map((language) => ({
		id: language.code,
		label: language.name,
		selected: locale.value === language.code,
		action: () => changeLanguage(language.code),
	})),
)
const currentLanguageName = computed(
	() =>
		LOCALES.find((language) => language.code === locale.value)?.name ??
		formatMessage(messages.language),
)

async function changeLanguage(newLocale: string) {
	await setLocale(newLocale)
	const settings = await getSettings()
	await setSettings({ ...settings, locale: newLocale })
}

const offline = ref(!navigator.onLine)

function handleOffline() {
	offline.value = true
}

function handleOnline() {
	offline.value = false
}

function handleQuickCreate(event: KeyboardEvent) {
	const target = event.target as HTMLElement | null
	if (
		event.key.toLowerCase() !== 'n' ||
		event.repeat ||
		event.metaKey ||
		event.ctrlKey ||
		event.altKey ||
		target?.isContentEditable ||
		['INPUT', 'TEXTAREA', 'SELECT'].includes(target?.tagName ?? '')
	) {
		return
	}

	if (!offline.value) {
		event.preventDefault()
		showCreationModal?.()
	}
}

onMounted(() => {
	window.addEventListener('offline', handleOffline)
	window.addEventListener('online', handleOnline)
	window.addEventListener('keydown', handleQuickCreate)
})

onUnmounted(() => {
	window.removeEventListener('offline', handleOffline)
	window.removeEventListener('online', handleOnline)
	window.removeEventListener('keydown', handleQuickCreate)
})
</script>

<template>
	<div class="flex flex-col min-h-full px-6 pb-6 pt-16">
		<div class="relative flex grow items-center justify-center">
			<div class="relative isolate flex flex-col items-center gap-6">
				<div class="size-[6.25rem]">
					<ModrinthIcon aria-hidden="true" class="pointer-events-none size-full" />
				</div>
				<div class="flex flex-col items-center gap-2">
					<h1 class="m-0 flex items-center gap-2 text-2xl font-semibold leading-8 text-contrast">
						{{ formatMessage(messages.welcomeTitle) }}
					</h1>
					<p class="m-0 text-center text-base leading-6 text-primary">
						{{ formatMessage(messages.welcomeDescription) }}
					</p>
				</div>
				<div class="flex w-72 flex-col items-center gap-4">
					<Button
						type="colored"
						color="brand"
						size="lg"
						class="!shadow-none"
						:disabled="offline"
						@click="showCreationModal?.()"
					>
						<PlusIcon />
						{{ formatMessage(messages.createInstance) }}
					</Button>
					<TeleportOverflowMenu
						:label="formatMessage(messages.language)"
						:options="languageOptions"
						:icon-only="false"
						placement="bottom-center"
					>
						<LanguagesIcon />
						{{ currentLanguageName }}
					</TeleportOverflowMenu>
				</div>
			</div>
		</div>
		<div
			class="flex flex-col h-max items-center justify-end gap-4 text-sm leading-5 text-secondary"
		>
			<span class="whitespace-nowrap">{{ formatMessage(messages.importPrompt) }}</span>
			<Button size="lg" class="!font-medium" :disabled="offline" @click="showImportModal?.()">
				<ImportIcon />
				{{ formatMessage(messages.importFromLauncher) }}
			</Button>
		</div>
	</div>
</template>
