<script setup lang="ts">
import { useId } from 'vue'

import enderPearlIcon from '../../../apps/app/icons/icon.ico?url'
import lightEnderPearlIcon from './ender-pearl-light.png?url'

const filterId = `ender-pearl-accent-${useId().replaceAll(':', '')}`
</script>

<template>
	<svg
		class="ender-pearl-graphic"
		xmlns="http://www.w3.org/2000/svg"
		viewBox="0 0 1 1"
		aria-hidden="true"
	>
		<defs>
			<filter
				:id="filterId"
				x="0"
				y="0"
				width="100%"
				height="100%"
				color-interpolation-filters="sRGB"
			>
				<feColorMatrix in="SourceGraphic" type="saturate" values="0" result="luminosity" />
				<feComponentTransfer in="luminosity" result="adjusted-luminosity">
					<feFuncR type="linear" slope="0.92" intercept="0.08" />
					<feFuncG type="linear" slope="0.92" intercept="0.08" />
					<feFuncB type="linear" slope="0.92" intercept="0.08" />
				</feComponentTransfer>
				<feFlood flood-color="var(--color-brand)" result="accent" />
				<feBlend in="accent" in2="adjusted-luminosity" mode="color" result="tinted" />
				<feFlood flood-color="var(--color-brand)" flood-opacity="0.04" result="accent-correction" />
				<feBlend in="accent-correction" in2="tinted" mode="normal" result="corrected" />
				<feComposite in="corrected" in2="SourceAlpha" operator="in" />
			</filter>
		</defs>
		<image
			class="pearl-accent"
			:href="enderPearlIcon"
			width="1"
			height="1"
			preserveAspectRatio="xMidYMid meet"
			:filter="`url(#${filterId})`"
		/>
		<image class="pearl-original pearl-original-dark" :href="enderPearlIcon" width="1" height="1" />
		<image
			class="pearl-original pearl-original-light"
			:href="lightEnderPearlIcon"
			x="-0.05"
			y="-0.05"
			width="1.1"
			height="1.1"
			preserveAspectRatio="xMidYMid meet"
		/>
	</svg>
</template>

<style scoped>
.ender-pearl-graphic {
	display: block;
}

.pearl-original {
	display: none;
}

.ender-pearl-graphic.default-icon .pearl-accent,
.ender-pearl-graphic.original .pearl-accent {
	display: none;
}

.ender-pearl-graphic.default-icon .pearl-original-dark,
.ender-pearl-graphic.original .pearl-original-dark {
	display: block;
}

:global(.disable-pearl-accent-adaptation .pearl-accent) {
	display: none;
}

:global(.disable-pearl-accent-adaptation .pearl-original-dark) {
	display: block;
}

:global(.light-mode.disable-pearl-accent-adaptation .pearl-original-dark) {
	display: none;
}

:global(.light-mode.disable-pearl-accent-adaptation .pearl-original-light) {
	display: block;
}
</style>
