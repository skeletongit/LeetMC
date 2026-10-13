<template>
	<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2700 593" role="img" aria-label="LeetMC">
		<image
			v-if="lightMode"
			class="ender-pearl-icon-light"
			:href="lightEnderPearlIcon"
			x="-81"
			y="-81"
			width="712"
			height="712"
		/>
		<image v-else :href="enderPearlIcon" width="593" height="593" />
		<text
			x="695"
			y="492"
			fill="currentColor"
			font-family="Inter, sans-serif"
			font-size="531"
			font-weight="600"
		>
			LeetMC
		</text>
	</svg>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'

import enderPearlIcon from '../../../../../apps/app/icons/icon.ico?url'
import lightEnderPearlIcon from '../../../../assets/branding/ender-pearl-light.png?url'

const lightMode = ref(
	typeof document !== 'undefined' && document.documentElement.classList.contains('light-mode'),
)
let themeObserver: MutationObserver | null = null

function updateTheme() {
	lightMode.value = document.documentElement.classList.contains('light-mode')
}

onMounted(() => {
	updateTheme()
	themeObserver = new MutationObserver(updateTheme)
	themeObserver.observe(document.documentElement, {
		attributeFilter: ['class'],
		attributes: true,
	})
})

onUnmounted(() => themeObserver?.disconnect())

defineProps({
	animate: {
		type: Boolean,
		default: false,
	},
})
</script>

<style scoped>
.ender-pearl-icon-light {
	image-rendering: pixelated;
}
</style>
