<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { onMounted, ref, watch } from 'vue'

import { download_skin_bytes } from '@/helpers/skins'

const props = withDefaults(
	defineProps<{
		src: string
		size?: number
	}>(),
	{
		size: 56,
	},
)

const canvasRef = ref<HTMLCanvasElement | null>(null)
const isLoading = ref(true)
const hasError = ref(false)

function drawHead(img: HTMLImageElement) {
	const canvas = canvasRef.value
	if (!canvas) return
	const ctx = canvas.getContext('2d')
	if (!ctx) return

	ctx.clearRect(0, 0, canvas.width, canvas.height)
	ctx.imageSmoothingEnabled = false

	// Draw head face (x: 8, y: 8, w: 8, h: 8)
	ctx.drawImage(img, 8, 8, 8, 8, 0, 0, canvas.width, canvas.height)
	// Draw outer hat/helmet layer on top (x: 40, y: 8, w: 8, h: 8)
	ctx.drawImage(img, 40, 8, 8, 8, 0, 0, canvas.width, canvas.height)

	isLoading.value = false
	hasError.value = false
}

async function load() {
	if (!props.src) {
		isLoading.value = false
		return
	}

	isLoading.value = true
	hasError.value = false

	const img = new Image()
	img.onload = () => {
		drawHead(img)
	}
	img.onerror = async () => {
		try {
			const bytes = await download_skin_bytes(props.src)
			const blob = new Blob([bytes], { type: 'image/png' })
			const blobUrl = URL.createObjectURL(blob)
			const fallbackImg = new Image()
			fallbackImg.onload = () => {
				drawHead(fallbackImg)
				URL.revokeObjectURL(blobUrl)
			}
			fallbackImg.onerror = () => {
				URL.revokeObjectURL(blobUrl)
				isLoading.value = false
				hasError.value = true
			}
			fallbackImg.src = blobUrl
		} catch {
			isLoading.value = false
			hasError.value = true
		}
	}
	img.src = props.src
}

onMounted(load)
watch(() => props.src, load)
</script>

<template>
	<div
		class="relative overflow-hidden rounded-lg bg-surface-4 flex items-center justify-center shrink-0"
		:style="{ width: `${size}px`, height: `${size}px` }"
	>
		<canvas
			ref="canvasRef"
			width="32"
			height="32"
			class="w-full h-full"
			style="image-rendering: pixelated"
		/>
		<SpinnerIcon
			v-if="isLoading"
			class="absolute size-4 animate-spin text-secondary pointer-events-none"
		/>
	</div>
</template>
