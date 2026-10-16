<script setup lang="ts" generic="T extends string">
import {
	EnderPearlIcon,
	GameIcon,
	MinecraftBlockIcon,
	MoonIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	SunIcon,
} from '@modrinth/assets'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

const { formatMessage } = useVIntl()

const { ariaLabel, modelValue, themeOptions, systemThemeColor, preferredDarkTheme } = defineProps<{
	ariaLabel: string
	modelValue: T
	themeOptions: readonly T[]
	systemThemeColor: T
	preferredDarkTheme: T
}>()

const emit = defineEmits<{
	'update:modelValue': [theme: T]
}>()

const themeLabels = defineMessages({
	system: {
		id: 'settings.display.theme.system',
		defaultMessage: 'Sync with system',
	},
	light: {
		id: 'settings.display.theme.light',
		defaultMessage: 'Light',
	},
	dark: {
		id: 'settings.display.theme.dark',
		defaultMessage: 'Dark',
	},
	oled: {
		id: 'settings.display.theme.oled',
		defaultMessage: 'Ender Pearl',
	},
	modrinth: {
		id: 'settings.display.theme.modrinth',
		defaultMessage: 'LeetMC',
	},
	retro: {
		id: 'settings.display.theme.retro',
		defaultMessage: 'Retro',
	},
})

const themeTooltips = defineMessages({
	preferredLight: {
		id: 'settings.display.theme.preferred-light-theme',
		defaultMessage: 'Preferred light theme',
	},
	preferredDark: {
		id: 'settings.display.theme.preferred-dark-theme',
		defaultMessage: 'Preferred dark theme',
	},
})

function formatTheme(theme: T): string {
	const message = themeLabels[theme as keyof typeof themeLabels]
	return message ? formatMessage(message) : theme
}

function getPreviewClass(option: T): string {
	const base = option === 'system' ? systemThemeColor : option
	return base.endsWith('-mode') ? base : `${base}-mode`
}
</script>

<template>
	<div class="theme-options" role="group" :aria-label="ariaLabel">
		<button
			v-for="option in themeOptions"
			:key="option"
			type="button"
			class="preview-radio button-base"
			:class="{ selected: modelValue === option }"
			:aria-pressed="modelValue === option"
			@click="emit('update:modelValue', option)"
		>
			<div class="preview" :class="getPreviewClass(option)" aria-hidden="true">
				<div class="example-card rounded-lg border border-solid border-surface-4 bg-surface-3">
					<div class="example-icon"></div>
					<div class="example-text-1"></div>
					<div class="example-text-2"></div>
				</div>
			</div>
			<div class="label">
				<RadioButtonCheckedIcon
					v-if="modelValue === option"
					class="radio shrink-0"
					aria-hidden="true"
				/>
				<RadioButtonIcon v-else class="radio shrink-0" aria-hidden="true" />
				{{ formatTheme(option) }}
				<SunIcon
					v-if="'light' === option"
					v-tooltip="formatMessage(themeTooltips.preferredLight)"
					class="theme-icon shrink-0"
					aria-hidden="true"
				/>
				<MoonIcon
					v-else-if="'dark' === option"
					v-tooltip="formatMessage(themeTooltips.preferredDark)"
					class="theme-icon shrink-0"
					aria-hidden="true"
				/>
				<EnderPearlIcon
					v-else-if="'oled' === option"
					class="theme-icon shrink-0 default-icon"
					aria-hidden="true"
				/>
				<GameIcon
					v-else-if="'retro' === option"
					class="theme-icon shrink-0"
					aria-hidden="true"
				/>
				<MinecraftBlockIcon
					v-else-if="'modrinth' === option || 'leetmc' === option"
					class="theme-icon shrink-0"
					aria-hidden="true"
				/>
			</div>
		</button>
	</div>
</template>

<style scoped lang="scss">
.theme-options {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
	gap: var(--gap-lg);

	.preview {
		&.light-mode {
			@extend .light-mode;
		}

		&.dark-mode {
			--surface-1: #000000;
			--surface-1-5: #050506;
			--surface-2: #09090a;
			--surface-2-5: #0c0d11;
			--surface-3: #101013;
			--surface-4: #1b1b20;
			--surface-5: #25262b;
			--color-bg: var(--surface-1);
			--color-raised-bg: var(--surface-3);
			--color-super-raised-bg: var(--surface-4);
			--color-button-bg: var(--surface-4);
			--color-base: #b0bac5;
			--color-secondary: #96a2b0;
		}

		&.oled-mode {
			--surface-1: #0b1211;
			--surface-1-5: #0e1715;
			--surface-2: #101a18;
			--surface-2-5: #13201d;
			--surface-3: #16231f;
			--surface-4: #1c302b;
			--surface-5: #29473f;
			--color-bg: var(--surface-1);
			--color-raised-bg: var(--surface-3);
			--color-super-raised-bg: var(--surface-4);
			--color-button-bg: var(--surface-4);
			--color-base: #b8c7c3;
			--color-secondary: #91a8a3;
		}

		&.modrinth-mode {
			--surface-1: #16181c;
			--surface-1-5: #1a1c20;
			--surface-2: #1d1f23;
			--surface-2-5: #222429;
			--surface-3: #27292e;
			--surface-4: #34363c;
			--surface-5: #42444a;
			--color-bg: var(--surface-1);
			--color-raised-bg: var(--surface-3);
			--color-super-raised-bg: var(--surface-4);
			--color-button-bg: var(--surface-4);
			--color-base: #b0bac5;
			--color-secondary: #96a2b0;
		}

		&.retro-mode {
			@extend .retro-mode;
		}
	}

	.preview .example-card {
		margin: 0;
		padding: 1rem;
		display: grid;
		grid-template: 'icon text1' 'icon text2';
		grid-template-columns: auto 1fr;
		gap: 0.5rem;
		outline: 2px solid transparent;

		.example-icon {
			grid-area: icon;
			width: 2rem;
			height: 2rem;
			background-color: var(--color-button-bg);
			border-radius: var(--radius-sm);
			outline: 2px solid transparent;
		}

		.example-text-1,
		.example-text-2 {
			height: 0.5rem;
			border-radius: var(--radius-sm);
			outline: 2px solid transparent;
		}

		.example-text-1 {
			grid-area: text1;
			width: 100%;
			background-color: var(--color-base);
		}

		.example-text-2 {
			grid-area: text2;
			width: 60%;
			background-color: var(--color-secondary);
		}
	}
}
</style>
