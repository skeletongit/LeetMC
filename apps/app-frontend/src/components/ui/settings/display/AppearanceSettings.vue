<script setup lang="ts">
import {
	AppearanceSettingsLayout,
	injectAuth,
	injectUserPreferences,
	provideAppearanceSettings,
	useSavable,
} from '@modrinth/ui'
import { platform } from '@tauri-apps/plugin-os'
import { computed, inject, onBeforeUnmount, onMounted, watch } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import {
	type ColorTheme,
	isDarkTheme,
	type Theme,
	type ThemeAccentColors,
	useTheme,
} from '@/composables/use-theme.ts'
import { type AppSettings, get, set } from '@/helpers/settings.ts'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'

const theme = useTheme()
const appSettings = useAppSettings()
const auth = injectAuth()
const { updatePreferences } = injectUserPreferences()
const settingsModal = inject(appSettingsModalContextKey, null)
const os = platform()

type AppearanceSettingsState = {
	theme: ColorTheme
	syncAcrossDevices: boolean
	advancedRendering: boolean
	adaptPearlToAccent: boolean
	accentColors: ThemeAccentColors
	liquidGlassIntensity: number
	nativeDecorations: boolean
}

function getAppearanceSettingsState(): AppearanceSettingsState {
	return {
		theme: theme.preferred,
		syncAcrossDevices: theme.syncAcrossDevices,
		advancedRendering: theme.advancedRendering,
		adaptPearlToAccent: theme.adaptPearlToAccent,
		accentColors: { ...theme.accentColors },
		liquidGlassIntensity: theme.liquidGlassIntensity,
		nativeDecorations: appSettings.nativeDecorations,
	}
}

const { saved, current, changes, saving, hasChanges, reset, save } = useSavable(
	getAppearanceSettingsState,
	async (appearanceChanges) => {
		const value = current.value
		if (
			value.syncAcrossDevices &&
			auth.user.value &&
			value.theme !== 'modrinth' &&
			(appearanceChanges.theme !== undefined || appearanceChanges.syncAcrossDevices !== undefined)
		) {
			await updatePreferences({
				appearance: value.theme === 'system' ? { auto: true } : { auto: false, theme: value.theme },
			})
		}

		const storedSettings = await get()
		const nextSettings: AppSettings = {
			...storedSettings,
			theme: value.theme,
			sync_theme_across_devices: value.syncAcrossDevices,
			advanced_rendering: value.advancedRendering,
			native_decorations: value.nativeDecorations,
		}

		await set(nextSettings)
		if (isDarkTheme(value.theme)) {
			theme.preferredDark = value.theme
		}
		theme.preferred = value.theme
		theme.syncAcrossDevices = value.syncAcrossDevices
		theme.advancedRendering = value.advancedRendering
		theme.adaptPearlToAccent = value.adaptPearlToAccent
		theme.accentColors = { ...value.accentColors }
		theme.liquidGlassIntensity = value.liquidGlassIntensity
		appSettings.nativeDecorations = value.nativeDecorations
	},
)

const themeOptions = computed(() => theme.options)

const preferredDarkTheme = computed(() =>
	isDarkTheme(current.value.theme) ? current.value.theme : theme.preferredDark,
)
const selectedAccentTheme = computed<Theme>(() => {
	if (current.value.theme !== 'system') return current.value.theme
	return theme.native === 'light' ? 'light' : preferredDarkTheme.value
})
const accentColor = computed(
	() =>
		current.value.accentColors[selectedAccentTheme.value] ??
		theme.defaultAccentColors[selectedAccentTheme.value],
)
const defaultAccentColor = computed(() => theme.defaultAccentColors[selectedAccentTheme.value])

function setTheme(value: ColorTheme): void {
	current.value.theme = value
}

function setSyncAcrossDevices(enabled: boolean): void {
	current.value.syncAcrossDevices = enabled
}

function setAdvancedRendering(enabled: boolean): void {
	current.value.advancedRendering = enabled
}

function setAdaptPearlToAccent(enabled: boolean): void {
	current.value.adaptPearlToAccent = enabled
}

function setAccentColor(color: string): void {
	current.value.accentColors = {
		...current.value.accentColors,
		[selectedAccentTheme.value]: color.toLowerCase(),
	}
}

function resetAccentColor(): void {
	const colors = { ...current.value.accentColors }
	delete colors[selectedAccentTheme.value]
	current.value.accentColors = colors
}

function setLiquidGlassIntensity(intensity: number): void {
	current.value.liquidGlassIntensity = intensity
}

function setNativeDecorations(enabled: boolean): void {
	current.value.nativeDecorations = enabled
}

watch(
	[() => current.value.theme, () => saved.value.theme],
	([selectedTheme, savedTheme]) => {
		theme.preview = selectedTheme === savedTheme ? null : selectedTheme
	},
	{ immediate: true },
)

watch(
	[() => current.value.liquidGlassIntensity, () => saved.value.liquidGlassIntensity],
	([selectedIntensity, savedIntensity]) => {
		theme.liquidGlassPreview = selectedIntensity === savedIntensity ? null : selectedIntensity
	},
	{ immediate: true },
)

watch(
	[() => current.value.accentColors, () => saved.value.accentColors],
	([selectedColors, savedColors]) => {
		const hasChanges = (Object.keys(theme.defaultAccentColors) as Theme[]).some(
			(themeName) => selectedColors[themeName] !== savedColors[themeName],
		)
		theme.accentColorsPreview = hasChanges ? { ...selectedColors } : null
	},
	{ deep: true, immediate: true },
)

watch(
	[() => current.value.adaptPearlToAccent, () => saved.value.adaptPearlToAccent],
	([selectedAdaptation, savedAdaptation]) => {
		theme.adaptPearlToAccentPreview =
			selectedAdaptation === savedAdaptation ? null : selectedAdaptation
	},
	{ immediate: true },
)

async function saveAppearanceSettings(): Promise<void> {
	try {
		await save()
	} catch {
		return
	}
}

onMounted(() => {
	settingsModal?.registerUnsavedChangesController({
		hasChanges: () => hasChanges.value,
		getOriginal: () => saved.value,
		getModified: () => changes.value,
		isSaving: () => saving.value,
		reset,
		save: saveAppearanceSettings,
	})
})

onBeforeUnmount(() => {
	theme.preview = null
	theme.liquidGlassPreview = null
	theme.accentColorsPreview = null
	theme.adaptPearlToAccentPreview = null
	settingsModal?.registerUnsavedChangesController(null)
})

provideAppearanceSettings({
	deferPersistence: true,
	theme: {
		current: computed(() => current.value.theme),
		options: themeOptions,
		system: computed(() => (theme.native === 'light' ? 'light' : preferredDarkTheme.value)),
		preferredDark: preferredDarkTheme,
		set: setTheme,
		syncAcrossDevices: {
			value: computed(() => current.value.syncAcrossDevices),
			set: setSyncAcrossDevices,
		},
		syncDisabled: computed(() => !auth.user.value),
	},
	advancedRendering: {
		value: computed(() => current.value.advancedRendering),
		set: setAdvancedRendering,
	},
	adaptPearlToAccent: {
		value: computed(() => current.value.adaptPearlToAccent),
		set: setAdaptPearlToAccent,
	},
	accentColor: {
		value: accentColor,
		defaultValue: defaultAccentColor,
		set: setAccentColor,
		reset: resetAccentColor,
	},
	liquidGlassIntensity: {
		value: computed(() => current.value.liquidGlassIntensity),
		set: setLiquidGlassIntensity,
	},
	nativeDecorations:
		os !== 'macos'
			? {
					value: computed(() => current.value.nativeDecorations),
					set: setNativeDecorations,
				}
			: undefined,
	updatePreferences,
})
</script>

<template>
	<AppearanceSettingsLayout />
</template>
