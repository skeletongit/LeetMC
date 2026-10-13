<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { onMounted, ref, watch } from 'vue'

import { download_skin_bytes } from '@/helpers/skins'

const props = withDefaults(
	defineProps<{
		src: string
		variant?: 'CLASSIC' | 'SLIM' | 'UNKNOWN'
		headOnly?: boolean
		height?: number
	}>(),
	{
		variant: 'CLASSIC',
		headOnly: false,
		height: 96,
	},
)

const canvasRef = ref<HTMLCanvasElement | null>(null)
const isLoading = ref(true)
const hasError = ref(false)

function drawHead(ctx: CanvasRenderingContext2D, img: HTMLImageElement, w: number, h: number) {
	ctx.clearRect(0, 0, w, h)
	ctx.imageSmoothingEnabled = false

	ctx.drawImage(img, 8, 8, 8, 8, 0, 0, w, h)
	ctx.drawImage(img, 40, 8, 8, 8, 0, 0, w, h)
}

function drawFullBody(ctx: CanvasRenderingContext2D, img: HTMLImageElement) {
	ctx.clearRect(0, 0, 64, 128)
	ctx.imageSmoothingEnabled = false

	const is64x64 = img.height >= 64
	const isSlim = props.variant === 'SLIM'
	const armW = isSlim ? 3 : 4
	const armDestW = isSlim ? 12 : 16
	const rightArmDestX = isSlim ? 4 : 0

	// 1. Head
	ctx.drawImage(img, 8, 8, 8, 8, 16, 0, 32, 32)
	ctx.drawImage(img, 40, 8, 8, 8, 16, 0, 32, 32)

	// 2. Torso
	ctx.drawImage(img, 20, 20, 8, 12, 16, 32, 32, 48)
	if (is64x64) {
		ctx.drawImage(img, 20, 36, 8, 12, 16, 32, 32, 48)
	}

	// 3. Right Arm
	ctx.drawImage(img, 44, 20, armW, 12, rightArmDestX, 32, armDestW, 48)
	if (is64x64) {
		ctx.drawImage(img, 44, 36, armW, 12, rightArmDestX, 32, armDestW, 48)
	}

	// 4. Left Arm
	if (is64x64) {
		ctx.drawImage(img, 36, 52, armW, 12, 48, 32, armDestW, 48)
		ctx.drawImage(img, 52, 52, armW, 12, 48, 32, armDestW, 48)
	} else {
		ctx.save()
		ctx.translate(48 + armDestW, 0)
		ctx.scale(-1, 1)
		ctx.drawImage(img, 44, 20, armW, 12, 0, 32, armDestW, 48)
		ctx.restore()
	}

	// 5. Right Leg
	ctx.drawImage(img, 4, 20, 4, 12, 16, 80, 16, 48)
	if (is64x64) {
		ctx.drawImage(img, 4, 36, 4, 12, 16, 80, 16, 48)
	}

	// 6. Left Leg
	if (is64x64) {
		ctx.drawImage(img, 20, 52, 4, 12, 32, 80, 16, 48)
		ctx.drawImage(img, 4, 52, 4, 12, 32, 80, 16, 48)
	} else {
		ctx.save()
		ctx.translate(32 + 16, 0)
		ctx.scale(-1, 1)
		ctx.drawImage(img, 4, 20, 4, 12, 0, 80, 16, 48)
		ctx.restore()
	}
}

function renderImage(img: HTMLImageElement) {
	const canvas = canvasRef.value
	if (!canvas) return
	const ctx = canvas.getContext('2d')
	if (!ctx) return

	if (props.headOnly) {
		drawHead(ctx, img, canvas.width, canvas.height)
	} else {
		drawFullBody(ctx, img)
	}

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
		renderImage(img)
	}
	img.onerror = async () => {
		try {
			const bytes = await download_skin_bytes(props.src)
			const blob = new Blob([bytes], { type: 'image/png' })
			const blobUrl = URL.createObjectURL(blob)
			const fallbackImg = new Image()
			fallbackImg.onload = () => {
				renderImage(fallbackImg)
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
watch([() => props.src, () => props.variant, () => props.headOnly], load)
</script>

<template>
	<div
		class="relative overflow-hidden rounded-lg bg-surface-4/30 flex items-center justify-center shrink-0"
		:style="
			headOnly
				? { width: `${height}px`, height: `${height}px` }
				: { width: `${Math.round(height / 2)}px`, height: `${height}px` }
		"
	>
		<canvas
			ref="canvasRef"
			:width="headOnly ? 32 : 64"
			:height="headOnly ? 32 : 128"
			class="h-full w-full object-contain"
			style="image-rendering: pixelated"
		/>
		<SpinnerIcon
			v-if="isLoading"
			class="absolute size-4 animate-spin text-secondary pointer-events-none"
		/>
	</div>
</template>
