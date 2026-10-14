import type { Skin } from '../skins'
import { download_skin_bytes, get_normalized_skin_texture } from '../skins'
import { headStorage } from '../storage/head-storage'

export async function generatePlayerHeadBlob(skinUrl: string, size: number = 64): Promise<Blob> {
	return new Promise((resolve, reject) => {
		const img = new Image()
		if (!skinUrl.startsWith('data:') && !skinUrl.startsWith('blob:')) {
			img.crossOrigin = 'anonymous'
		}

		const renderHead = (sourceImg: HTMLImageElement) => {
			try {
				const outputCanvas = document.createElement('canvas')
				const outputCtx = outputCanvas.getContext('2d')

				if (!outputCtx) {
					throw new Error('Could not get 2D context from output canvas')
				}

				outputCanvas.width = size
				outputCanvas.height = size
				outputCtx.imageSmoothingEnabled = false

				// Base head layer: source rectangle (8, 8, 8, 8)
				outputCtx.drawImage(sourceImg, 8, 8, 8, 8, 0, 0, size, size)

				// Hat / outer accessory layer: source rectangle (40, 8, 8, 8)
				outputCtx.drawImage(sourceImg, 40, 8, 8, 8, 0, 0, size, size)

				outputCanvas.toBlob(
					(blob) => {
						if (blob) {
							resolve(blob)
						} else {
							reject(new Error('Failed to create blob from canvas'))
						}
					},
					'image/png',
				)
			} catch (error) {
				reject(error)
			}
		}

		img.onload = () => {
			renderHead(img)
		}

		img.onerror = async () => {
			if (!skinUrl.startsWith('data:') && !skinUrl.startsWith('blob:')) {
				try {
					const bytes = await download_skin_bytes(skinUrl)
					const blob = new Blob([bytes], { type: 'image/png' })
					const blobUrl = URL.createObjectURL(blob)
					const fallbackImg = new Image()
					fallbackImg.onload = () => {
						renderHead(fallbackImg)
						URL.revokeObjectURL(blobUrl)
					}
					fallbackImg.onerror = () => {
						URL.revokeObjectURL(blobUrl)
						reject(new Error('Failed to load fallback skin image'))
					}
					fallbackImg.src = blobUrl
					return
				} catch (downloadErr) {
					reject(downloadErr)
					return
				}
			}
			reject(new Error('Failed to load skin texture image'))
		}

		img.src = skinUrl
	})
}

export async function getPlayerHeadUrl(skin: Skin, forceRefresh = false): Promise<string> {
	const key = `${skin.texture_key}-head`
	if (!forceRefresh) {
		const cached = await headStorage.retrieve(key).catch(() => null)
		if (cached) return cached
	}
	let blob: Blob
	if (skin.texture) {
		try {
			blob = await generatePlayerHeadBlob(skin.texture, 64)
		} catch {
			const normalized = await get_normalized_skin_texture(skin)
			blob = await generatePlayerHeadBlob(normalized, 64)
		}
	} else {
		const normalized = await get_normalized_skin_texture(skin)
		blob = await generatePlayerHeadBlob(normalized, 64)
	}
	await headStorage.store(key, blob).catch(() => {})
	return URL.createObjectURL(blob)
}
