<template>
	<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2700 593" role="img" aria-label="LeetMC">
		<EnderPearlGraphic width="593" height="593" />
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
import EnderPearlGraphic from '../../../../assets/branding/EnderPearlGraphic.vue'

defineProps({
	animate: {
		type: Boolean,
		default: false,
	},
})
</script>
