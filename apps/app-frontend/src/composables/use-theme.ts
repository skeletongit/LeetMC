import { prepareThemeColorTransition } from '@modrinth/ui'
import { computed, reactive, ref, watch } from 'vue'

export const THEME_OPTIONS = ['dark', 'light', 'oled', 'modrinth', 'retro', 'system'] as const
export const DARK_THEMES = ['dark', 'oled', 'modrinth', 'retro'] as const

export type ColorTheme = (typeof THEME_OPTIONS)[number]
export type DarkTheme = (typeof DARK_THEMES)[number]
export type Theme = Exclude<ColorTheme, 'system'>
export type ThemeAccentColors = Partial<Record<Theme, string>>
type NativeTheme = 'light' | 'dark'

export const DEFAULT_THEME_ACCENT_COLORS: Record<Theme, string> = {
	dark: '#2ccdb1',
	light: '#d3324e',
	oled: '#2ccdb1',
	modrinth: '#1bd96a',
	retro: '#4d9227',
}

const PREFERRED_THEME_KEY = 'modrinth-theme'
const PREFERRED_DARK_THEME_KEY = 'modrinth-preferred-dark-theme'
const LIQUID_GLASS_INTENSITY_KEY = 'leetmc-liquid-glass-intensity'
const THEME_ACCENT_COLORS_KEY = 'leetmc-theme-accent-colors'
const ADAPT_PEARL_TO_ACCENT_KEY = 'leetmc-adapt-pearl-to-accent'
const HEX_COLOR_PATTERN = /^#[0-9a-f]{6}$/i

export function isDarkTheme(theme: string): theme is DarkTheme {
	return (DARK_THEMES as readonly string[]).includes(theme)
}

function loadPreferredTheme(): ColorTheme {
	try {
		const stored = window.localStorage.getItem(PREFERRED_THEME_KEY)
		if (stored && (THEME_OPTIONS as readonly string[]).includes(stored)) {
			return stored as ColorTheme
		}
	} catch {
		// storage blocked or full
	}

	for (const option of THEME_OPTIONS) {
		if (option !== 'system' && document.documentElement.classList.contains(`${option}-mode`)) {
			return option
		}
	}

	return 'dark'
}

function loadPreferredDarkTheme(): DarkTheme {
	try {
		const stored = window.localStorage.getItem(PREFERRED_DARK_THEME_KEY)
		if (stored && isDarkTheme(stored)) {
			return stored
		}
	} catch {
		// storage blocked or full
	}

	return 'dark'
}

function loadLiquidGlassIntensity(): number {
	try {
		const rawStored = window.localStorage.getItem(LIQUID_GLASS_INTENSITY_KEY)
		const stored = rawStored === null ? Number.NaN : Number(rawStored)
		if (Number.isFinite(stored)) {
			return Math.max(0, Math.min(100, stored))
		}
	} catch {
		// storage blocked or full
	}

	return 100
}

function loadAdaptPearlToAccent(): boolean {
	try {
		return window.localStorage.getItem(ADAPT_PEARL_TO_ACCENT_KEY) !== 'false'
	} catch {
		return true
	}
}

function loadThemeAccentColors(): ThemeAccentColors {
	try {
		const stored = JSON.parse(window.localStorage.getItem(THEME_ACCENT_COLORS_KEY) ?? '{}')
		if (!stored || typeof stored !== 'object' || Array.isArray(stored)) return {}

		const colors: ThemeAccentColors = {}
		for (const theme of Object.keys(DEFAULT_THEME_ACCENT_COLORS) as Theme[]) {
			const color = stored[theme]
			if (typeof color === 'string' && HEX_COLOR_PATTERN.test(color)) {
				colors[theme] = color.toLowerCase()
			}
		}
		return colors
	} catch {
		return {}
	}
}

function hexToRgb(color: string): [number, number, number] {
	return [
		Number.parseInt(color.slice(1, 3), 16),
		Number.parseInt(color.slice(3, 5), 16),
		Number.parseInt(color.slice(5, 7), 16),
	]
}

function getAccentContrastColor([red, green, blue]: [number, number, number]): string {
	const linearize = (channel: number) => {
		const normalized = channel / 255
		return normalized <= 0.04045 ? normalized / 12.92 : ((normalized + 0.055) / 1.055) ** 2.4
	}
	const luminance = 0.2126 * linearize(red) + 0.7152 * linearize(green) + 0.0722 * linearize(blue)

	return luminance > 0.179 ? '#0a0b0c' : '#ffffff'
}

const preferred = ref<ColorTheme>(loadPreferredTheme())
const preview = ref<ColorTheme | null>(null)
const preferredDark = ref<DarkTheme>(loadPreferredDarkTheme())
const liquidGlassIntensity = ref(loadLiquidGlassIntensity())
const liquidGlassPreview = ref<number | null>(null)
const accentColors = ref<ThemeAccentColors>(loadThemeAccentColors())
const accentColorsPreview = ref<ThemeAccentColors | null>(null)
const advancedRendering = ref(true)
const adaptPearlToAccent = ref(loadAdaptPearlToAccent())
const adaptPearlToAccentPreview = ref<boolean | null>(null)
const activeAdaptPearlToAccent = computed(
	() => adaptPearlToAccentPreview.value ?? adaptPearlToAccent.value,
)
const syncAcrossDevices = ref(false)
const nativeThemeQuery = window.matchMedia('(prefers-color-scheme: dark)')
const native = ref<NativeTheme>(nativeThemeQuery.matches ? 'dark' : 'light')
const active = computed<Theme>(() => {
	const selectedTheme = preview.value ?? preferred.value
	if (selectedTheme !== 'system') {
		return selectedTheme
	}

	return native.value === 'light' ? 'light' : preferredDark.value
})
const activeLiquidGlassIntensity = computed(
	() => liquidGlassPreview.value ?? liquidGlassIntensity.value,
)
const activeAccentColor = computed(
	() => (accentColorsPreview.value ?? accentColors.value)[active.value],
)

nativeThemeQuery.addEventListener('change', (event) => {
	native.value = event.matches ? 'dark' : 'light'
})

watch([preferred, preview], ([selectedPreferred, selectedPreview]) => {
	const selectedTheme = selectedPreview ?? selectedPreferred
	if (isDarkTheme(selectedTheme)) {
		preferredDark.value = selectedTheme
	}
})

watch(
	preferred,
	(theme) => {
		try {
			window.localStorage.setItem(PREFERRED_THEME_KEY, theme)
		} catch {
			// storage blocked or full
		}
	},
	{ immediate: true },
)

watch(preferredDark, (theme) => {
	try {
		window.localStorage.setItem(PREFERRED_DARK_THEME_KEY, theme)
	} catch {
		// storage blocked or full
	}
})

watch(
	liquidGlassIntensity,
	(intensity) => {
		try {
			window.localStorage.setItem(LIQUID_GLASS_INTENSITY_KEY, String(intensity))
		} catch {
			// storage blocked or full
		}
	},
	{ immediate: true },
)

watch(
	accentColors,
	(colors) => {
		try {
			window.localStorage.setItem(THEME_ACCENT_COLORS_KEY, JSON.stringify(colors))
		} catch {
			// storage blocked or full
		}
	},
	{ deep: true, immediate: true },
)

watch(
	adaptPearlToAccent,
	(enabled) => {
		try {
			window.localStorage.setItem(ADAPT_PEARL_TO_ACCENT_KEY, String(enabled))
		} catch {
			// storage blocked or full
		}
	},
	{ immediate: true },
)

watch(
	activeAdaptPearlToAccent,
	(enabled) => {
		document.documentElement.classList.toggle('disable-pearl-accent-adaptation', !enabled)
	},
	{ immediate: true },
)

watch(
	activeLiquidGlassIntensity,
	(intensity) => {
		document.documentElement.style.setProperty('--liquid-glass-level', String(intensity / 100))
	},
	{ immediate: true },
)

watch(
	[active, activeAccentColor],
	([, color]) => {
		const html = document.documentElement
		const customProperties = [
			'--color-brand',
			'--color-brand-highlight',
			'--color-brand-shadow',
			'--color-accent-contrast',
			'--color-button-text-selected',
			'--color-focus-ring',
			'--loading-bar-gradient',
		]

		for (const property of customProperties) {
			html.style.removeProperty(property)
		}
		if (!color) return

		const [red, green, blue] = hexToRgb(color)
		html.style.setProperty('--color-brand', color)
		html.style.setProperty('--color-brand-highlight', `rgb(${red} ${green} ${blue} / 25%)`)
		html.style.setProperty('--color-brand-shadow', `rgb(${red} ${green} ${blue} / 70%)`)
		html.style.setProperty('--color-accent-contrast', getAccentContrastColor([red, green, blue]))
		html.style.setProperty('--color-button-text-selected', color)
		html.style.setProperty('--color-focus-ring', color)
		html.style.setProperty(
			'--loading-bar-gradient',
			`linear-gradient(to right, ${color} 0%, color-mix(in srgb, ${color} 55%, white) 100%)`,
		)
	},
	{ immediate: true },
)

watch(
	active,
	(theme, previousTheme) => {
		if (previousTheme && previousTheme !== theme) {
			prepareThemeColorTransition()
		}

		const html = document.documentElement
		for (const option of THEME_OPTIONS) {
			html.classList.remove(`${option}-mode`)
		}
		html.classList.add(`${theme}-mode`)
	},
	{ immediate: true },
)

function applyAccountAppearance(appearance: { auto: boolean; theme: string }): void {
	if (isDarkTheme(appearance.theme)) {
		preferredDark.value = appearance.theme
	}

	if (appearance.auto) {
		preferred.value = 'system'
		return
	}

	if ((THEME_OPTIONS as readonly string[]).includes(appearance.theme)) {
		preferred.value = appearance.theme as ColorTheme
	}
}

const theme = reactive({
	preferred,
	preview,
	preferredDark,
	liquidGlassIntensity,
	liquidGlassPreview,
	accentColors,
	accentColorsPreview,
	defaultAccentColors: DEFAULT_THEME_ACCENT_COLORS,
	active,
	native,
	syncAcrossDevices,
	advancedRendering,
	adaptPearlToAccent,
	adaptPearlToAccentPreview,
	options: THEME_OPTIONS,
	applyAccountAppearance,
})

export function useTheme() {
	return theme
}
