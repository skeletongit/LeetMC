<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'

import enderPearlIcon from '../../../apps/app/icons/icon.ico?url'
import lightEnderPearlIcon from './ender-pearl-light.png?url'

const lightMode = ref(
	typeof document !== 'undefined' && document.documentElement.classList.contains('light-mode'),
)
let themeObserver = null

function updateTheme() {
	lightMode.value = document.documentElement.classList.contains('light-mode')
}

onMounted(() => {
	updateTheme()
	themeObserver = new MutationObserver(updateTheme)
	themeObserver.observe(document.documentElement, { attributeFilter: ['class'] })
})

onUnmounted(() => themeObserver?.disconnect())
</script>

<template>
	<img
		:src="lightMode ? lightEnderPearlIcon : enderPearlIcon"
		alt=""
		:class="{ 'ender-pearl-icon-light': lightMode }"
	/>
</template>

<style scoped>
.ender-pearl-icon-light {
	transform: translate(-3.75%, -3.75%) scale(1.2);
	image-rendering: pixelated;
}
</style>
