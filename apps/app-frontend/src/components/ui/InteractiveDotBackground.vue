<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'

const props = withDefaults(
	defineProps<{
		animated?: boolean
		rightInset?: number
	}>(),
	{
		animated: true,
		rightInset: 0,
	},
)

const canvas = ref<HTMLCanvasElement | null>(null)

const DOT_SPACING = 22
const DOT_RADIUS = 0.7
const EFFECT_RADIUS = 150
const MAX_DISPLACEMENT = 16
const MAX_DEVICE_PIXEL_RATIO = 1.5

let context: CanvasRenderingContext2D | null = null
let resizeObserver: ResizeObserver | null = null
let reducedMotionQuery: MediaQueryList | null = null
let animationFrame: number | null = null
let targetX = 0
let targetY = 0
let currentX = 0
let currentY = 0
let targetStrength = 0
let currentStrength = 0

function draw() {
	const element = canvas.value
	if (!element || !context) return

	const width = element.clientWidth
	const height = element.clientHeight
	if (width === 0 || height === 0) return
	const styles = getComputedStyle(element)
	const baseColor = styles.getPropertyValue('--dot-base-color').trim() || styles.color
	const activeColor = styles.getPropertyValue('--dot-active-color').trim() || styles.color
	const configuredBaseOpacity = Number.parseFloat(styles.getPropertyValue('--dot-base-opacity'))
	const baseOpacity = Number.isNaN(configuredBaseOpacity) ? 0.14 : configuredBaseOpacity

	context.clearRect(0, 0, width, height)
	context.fillStyle = baseColor
	context.globalAlpha = baseOpacity
	context.beginPath()

	for (let x = DOT_SPACING / 2; x < width; x += DOT_SPACING) {
		for (let y = DOT_SPACING / 2; y < height; y += DOT_SPACING) {
			if (currentStrength > 0.01 && Math.hypot(x - currentX, y - currentY) < EFFECT_RADIUS) {
				continue
			}

			context.moveTo(x + DOT_RADIUS, y)
			context.arc(x, y, DOT_RADIUS, 0, Math.PI * 2)
		}
	}

	context.fill()

	if (currentStrength <= 0.01) {
		context.globalAlpha = 1
		return
	}

	const startX = Math.max(
		DOT_SPACING / 2,
		Math.floor((currentX - EFFECT_RADIUS) / DOT_SPACING) * DOT_SPACING + DOT_SPACING / 2,
	)
	const endX = Math.min(width, currentX + EFFECT_RADIUS)
	const startY = Math.max(
		DOT_SPACING / 2,
		Math.floor((currentY - EFFECT_RADIUS) / DOT_SPACING) * DOT_SPACING + DOT_SPACING / 2,
	)
	const endY = Math.min(height, currentY + EFFECT_RADIUS)
	context.fillStyle = activeColor

	for (let x = startX; x <= endX; x += DOT_SPACING) {
		for (let y = startY; y <= endY; y += DOT_SPACING) {
			const distance = Math.hypot(x - currentX, y - currentY)
			if (distance >= EFFECT_RADIUS) continue

			const proximity = 1 - distance / EFFECT_RADIUS
			const easedProximity = proximity * proximity * (3 - 2 * proximity) * currentStrength
			const directionX = distance > 0 ? (x - currentX) / distance : 1
			const directionY = distance > 0 ? (y - currentY) / distance : 0
			const displacement = easedProximity * MAX_DISPLACEMENT
			const displacedX = x + directionX * displacement
			const displacedY = y + directionY * displacement
			context.globalAlpha = 0.16 + easedProximity * 0.68
			context.beginPath()
			context.arc(displacedX, displacedY, DOT_RADIUS + easedProximity * 1.7, 0, Math.PI * 2)
			context.fill()
		}
	}

	context.globalAlpha = 1
}

function resize() {
	const element = canvas.value
	if (!element || !context) return

	const ratio = Math.min(window.devicePixelRatio || 1, MAX_DEVICE_PIXEL_RATIO)
	const width = Math.round(element.clientWidth * ratio)
	const height = Math.round(element.clientHeight * ratio)

	if (element.width !== width || element.height !== height) {
		element.width = width
		element.height = height
		context.setTransform(ratio, 0, 0, ratio, 0, 0)
	}

	draw()
}

function animate() {
	animationFrame = null
	currentX += (targetX - currentX) * 0.22
	currentY += (targetY - currentY) * 0.22
	currentStrength += (targetStrength - currentStrength) * 0.16
	draw()

	const positionSettled = Math.abs(targetX - currentX) + Math.abs(targetY - currentY) < 0.2
	const strengthSettled = Math.abs(targetStrength - currentStrength) < 0.01

	if (!positionSettled || !strengthSettled) {
		animationFrame = requestAnimationFrame(animate)
	} else {
		currentX = targetX
		currentY = targetY
		currentStrength = targetStrength
		draw()
	}
}

function requestDraw() {
	if (animationFrame === null) {
		animationFrame = requestAnimationFrame(animate)
	}
}

function canAnimate() {
	return props.animated && !reducedMotionQuery?.matches && !document.hidden
}

function handlePointerMove(event: PointerEvent) {
	const element = canvas.value
	if (!element || !canAnimate()) return

	const bounds = element.getBoundingClientRect()
	const inside =
		event.clientX >= bounds.left &&
		event.clientX <= bounds.right &&
		event.clientY >= bounds.top &&
		event.clientY <= bounds.bottom

	if (inside) {
		targetX = event.clientX - bounds.left
		targetY = event.clientY - bounds.top
		targetStrength = 1
	} else {
		targetStrength = 0
	}

	requestDraw()
}

function resetEffect() {
	targetStrength = 0
	requestDraw()
}

function clearEffect() {
	targetStrength = 0
	currentStrength = 0
	draw()
}

function handleAnimationPreferenceChange() {
	if (!canAnimate()) clearEffect()
}

function handleVisibilityChange() {
	if (document.hidden && animationFrame !== null) {
		cancelAnimationFrame(animationFrame)
		animationFrame = null
	}

	clearEffect()
}

watch(
	() => props.animated,
	(animated) => {
		if (!animated) clearEffect()
	},
)

watch(
	() => props.rightInset,
	() => requestAnimationFrame(resize),
)

onMounted(() => {
	const element = canvas.value
	if (!element) return

	context = element.getContext('2d')
	if (!context) return

	reducedMotionQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
	resizeObserver = new ResizeObserver(resize)
	resizeObserver.observe(element)
	window.addEventListener('pointermove', handlePointerMove, { passive: true })
	window.addEventListener('pointerleave', resetEffect)
	document.addEventListener('visibilitychange', handleVisibilityChange)
	reducedMotionQuery.addEventListener('change', handleAnimationPreferenceChange)
	resize()
})

onUnmounted(() => {
	if (animationFrame !== null) cancelAnimationFrame(animationFrame)
	resizeObserver?.disconnect()
	window.removeEventListener('pointermove', handlePointerMove)
	window.removeEventListener('pointerleave', resetEffect)
	document.removeEventListener('visibilitychange', handleVisibilityChange)
	reducedMotionQuery?.removeEventListener('change', handleAnimationPreferenceChange)
})
</script>

<template>
	<canvas
		ref="canvas"
		aria-hidden="true"
		class="interactive-dot-background"
		:style="{ width: `calc(100% - ${rightInset}px)` }"
	/>
</template>

<style scoped>
.interactive-dot-background {
	--dot-base-color: var(--color-text-primary);
	--dot-active-color: var(--color-text-primary);
	--dot-base-opacity: 0.14;

	position: absolute;
	top: 0;
	bottom: 0;
	left: 0;
	height: 100%;
	z-index: -1;
	pointer-events: none;
	color: var(--color-text-primary);
}

:global(html.light-mode) .interactive-dot-background {
	--dot-base-color: var(--color-text-tertiary);
	--dot-active-color: var(--color-brand);
	--dot-base-opacity: 0.22;
}
</style>
