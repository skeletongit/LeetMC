<script setup lang="ts">
import Toggle from '#ui/components/base/Toggle.vue'
import Slider from '#ui/components/base/Slider.vue'
import Button from '#ui/components/base/buttons/Button.vue'
import ColorPicker from '#ui/components/base/inputs/ColorPicker.vue'
import { useVIntl } from '#ui/composables'

import AppearanceSettingsThemeSelector from './components/appearance-settings-theme-selector.vue'
import AppearanceSettingRow from './components/AppearanceSettingRow.vue'
import ProjectLayoutSelector from './components/ProjectLayoutSelector.vue'
import { appearanceSettingsMessages as messages } from './messages'
import { injectAppearanceSettings } from './providers'

const appearance = injectAppearanceSettings()
const { formatMessage } = useVIntl()

const theme = appearance.theme
const currentTheme = theme.current
const themeOptions = theme.options
const systemTheme = theme.system
const preferredDarkTheme = theme.preferredDark
const advancedRendering = appearance.advancedRendering.value
const accentColor = appearance.accentColor
const accentColorValue = accentColor?.value
const defaultAccentColor = accentColor?.defaultValue
const liquidGlassIntensity = appearance.liquidGlassIntensity
const liquidGlassIntensityValue = liquidGlassIntensity?.value
const nativeDecorations = appearance.nativeDecorations
const nativeDecorationsValue = nativeDecorations?.value
const projectLayouts = appearance.projectLayouts
const projectLayoutValues = projectLayouts?.value
const externalLinksNewTab = appearance.externalLinksNewTab
const externalLinksNewTabValue = externalLinksNewTab?.value
const sidebarPreferences = appearance.sidebarPreferences
const sidebarPreferenceValues = sidebarPreferences?.value
</script>

<template>
	<div>
		<section>
			<div class="flex flex-col gap-1">
				<h2 class="m-0 text-xl font-semibold text-contrast">
					{{ formatMessage(messages.colorThemeTitle) }}
				</h2>
				<p class="m-0 text-secondary">
					{{ formatMessage(messages.colorThemeDescription) }}
				</p>
			</div>

			<AppearanceSettingsThemeSelector
				class="mt-4"
				:aria-label="formatMessage(messages.colorThemeTitle)"
				:model-value="currentTheme"
				:theme-options="themeOptions"
				:system-theme-color="systemTheme"
				:preferred-dark-theme="preferredDarkTheme"
				@update:model-value="theme.update"
			/>

			<div v-if="accentColor" class="mt-6">
				<h3 id="accent-color-label" class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.accentColorTitle) }}
				</h3>
				<p class="mb-3 mt-1 text-secondary">
					{{ formatMessage(messages.accentColorDescription) }}
				</p>
				<div class="flex flex-wrap items-center gap-3">
					<ColorPicker
						:model-value="accentColorValue ?? '#000000'"
						:label="formatMessage(messages.accentColorTitle)"
						@update:model-value="accentColor.update"
					/>
					<code class="rounded-lg bg-surface-3 px-3 py-2 font-mono text-sm uppercase text-contrast">
						{{ accentColorValue }}
					</code>
					<Button
						size="sm"
						:disabled="accentColorValue === defaultAccentColor"
						@click="accentColor.reset"
					>
						{{ formatMessage(messages.resetAccentColor) }}
					</Button>
				</div>
			</div>

			<div v-if="liquidGlassIntensity" class="mt-6">
				<h3 id="liquid-glass-intensity-label" class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.liquidGlassTitle) }}
				</h3>
				<p class="mb-2 mt-1 text-secondary">
					{{ formatMessage(messages.liquidGlassDescription) }}
				</p>
				<Slider
					:model-value="liquidGlassIntensityValue ?? 100"
					:min="0"
					:max="100"
					:step="1"
					:snap-points="[0, 25, 50, 75, 100]"
					:snap-range="4"
					min-label="0%"
					max-label="100%"
					:aria-label="formatMessage(messages.liquidGlassTitle)"
					@update:model-value="liquidGlassIntensity.update"
				/>
			</div>
		</section>

		<section v-if="projectLayouts" class="mt-8 border-0 border-t border-solid border-divider pt-6">
			<div class="flex flex-col gap-1">
				<h2 class="m-0 text-xl font-semibold text-contrast">
					{{ formatMessage(messages.projectListLayoutsTitle) }}
				</h2>
				<p class="m-0 text-secondary">
					{{ formatMessage(messages.projectListLayoutsDescription) }}
				</p>
			</div>
			<div class="mt-4 flex flex-col gap-6">
				<ProjectLayoutSelector
					v-for="projectLayout in projectLayoutValues"
					:key="projectLayout.type"
					:title="formatMessage(messages[projectLayout.type])"
					:model-value="projectLayout.layout"
					@update:model-value="projectLayouts.update(projectLayout.type, $event)"
				/>
			</div>
		</section>

		<div class="mt-8 border-0 border-t border-solid border-divider pt-6">
			<div class="flex flex-col gap-6">
				<AppearanceSettingRow
					control-id="advanced-rendering"
					:title="formatMessage(messages.advancedRenderingTitle)"
					:description="formatMessage(messages.advancedRenderingDescription)"
				>
					<template #default="{ labelledBy }">
						<Toggle
							id="advanced-rendering"
							:model-value="advancedRendering"
							:aria-labelledby="labelledBy"
							@update:model-value="appearance.advancedRendering.update"
						/>
					</template>
				</AppearanceSettingRow>

				<AppearanceSettingRow
					v-if="externalLinksNewTab"
					control-id="external-links-new-tab"
					:title="formatMessage(messages.externalLinksNewTabTitle)"
					:description="formatMessage(messages.externalLinksNewTabDescription)"
				>
					<template #default="{ labelledBy }">
						<Toggle
							id="external-links-new-tab"
							:model-value="externalLinksNewTabValue ?? false"
							:aria-labelledby="labelledBy"
							@update:model-value="externalLinksNewTab.update"
						/>
					</template>
				</AppearanceSettingRow>

				<template v-if="sidebarPreferences">
					<AppearanceSettingRow
						control-id="search-layout-toggle"
						:title="formatMessage(messages.rightAlignedFiltersSidebarTitle)"
						:description="formatMessage(messages.rightAlignedFiltersSidebarDescription)"
					>
						<template #default="{ labelledBy }">
							<Toggle
								id="search-layout-toggle"
								:model-value="sidebarPreferenceValues?.right_aligned_search ?? false"
								:aria-labelledby="labelledBy"
								@update:model-value="sidebarPreferences.update('right_aligned_search', $event)"
							/>
						</template>
					</AppearanceSettingRow>

					<AppearanceSettingRow
						control-id="project-layout-toggle"
						:title="formatMessage(messages.leftAlignedContentSidebarTitle)"
						:description="formatMessage(messages.leftAlignedContentSidebarDescription)"
					>
						<template #default="{ labelledBy }">
							<Toggle
								id="project-layout-toggle"
								:model-value="sidebarPreferenceValues?.left_aligned_content ?? false"
								:aria-labelledby="labelledBy"
								@update:model-value="sidebarPreferences.update('left_aligned_content', $event)"
							/>
						</template>
					</AppearanceSettingRow>
				</template>

				<AppearanceSettingRow
					v-if="nativeDecorations"
					control-id="native-decorations"
					:title="formatMessage(messages.nativeDecorationsTitle)"
					:description="formatMessage(messages.nativeDecorationsDescription)"
				>
					<template #default="{ labelledBy }">
						<Toggle
							id="native-decorations"
							:model-value="nativeDecorationsValue ?? false"
							:aria-labelledby="labelledBy"
							@update:model-value="nativeDecorations.update"
						/>
					</template>
				</AppearanceSettingRow>
			</div>
		</div>
	</div>
</template>
