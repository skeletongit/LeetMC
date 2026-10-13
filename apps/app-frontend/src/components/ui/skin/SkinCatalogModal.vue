<script setup lang="ts">
import {
	CheckIcon,
	DownloadIcon,
	GlobeIcon,
	SaveIcon,
	SearchIcon,
	ShirtIcon,
	SpinnerIcon,
	UserSearchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	NewModal,
	RadioButtons,
	SkinPreviewRenderer,
	useVIntl,
} from '@modrinth/ui'
import { arrayBufferToBase64 } from '@modrinth/utils/utils'
import { computed, ref } from 'vue'

import {
	add_and_equip_custom_skin,
	type CatalogSearchResult,
	download_skin_bytes,
	save_custom_skin,
	search_catalog_skins,
	search_player_skin,
	type SearchedSkin,
	type Skin,
	type SkinModel,
} from '@/helpers/skins'

const emit = defineEmits<{
	(e: 'equipped', skin: Skin): void
	(e: 'saved', skin: Skin): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()

const messages = defineMessages({
	catalogModalTitle: {
		id: 'app.skins.catalog.title',
		defaultMessage: 'Skin Catalog (NameMC)',
	},
	catalogTab: {
		id: 'app.skins.catalog.tab.catalog',
		defaultMessage: 'Catalog & Trending',
	},
	playerTab: {
		id: 'app.skins.catalog.tab.player',
		defaultMessage: 'Player Lookup',
	},
	catalogSearchPlaceholder: {
		id: 'app.skins.catalog.search.placeholder',
		defaultMessage: 'Search by tags (anime, creeper, knight, boy, pvp...)',
	},
	tagAll: {
		id: 'app.skins.catalog.tags.all',
		defaultMessage: 'All',
	},
	noSkinsFound: {
		id: 'app.skins.catalog.empty',
		defaultMessage: 'No skins found. Try searching for a different tag.',
	},
	pageInfo: {
		id: 'app.skins.catalog.pagination.info',
		defaultMessage: 'Page {page} of {totalPages}',
	},
	playerSearchPlaceholder: {
		id: 'app.skins.catalog.player-search.placeholder',
		defaultMessage: 'Enter player username (e.g. Notch, Dream) or NameMC link',
	},
	findSkinButton: {
		id: 'app.skins.catalog.player-search.button',
		defaultMessage: 'Find skin',
	},
	playerSearchingNotice: {
		id: 'app.skins.catalog.player-search.searching',
		defaultMessage: 'Looking up Mojang & NameMC...',
	},
	modelLabel: {
		id: 'app.skins.catalog.player-search.model',
		defaultMessage: 'Model: {model}',
	},
	playerSkinFoundNotice: {
		id: 'app.skins.catalog.player-search.found',
		defaultMessage: 'Skin found and ready to equip!',
	},
	playerNotFound: {
		id: 'app.skins.catalog.player-search.not-found',
		defaultMessage: 'Player not found or has no custom skin.',
	},
	playerSearchHint: {
		id: 'app.skins.catalog.player-search.hint',
		defaultMessage: 'Enter any player username to instantly load their skin from Minecraft!',
	},
	previewTitle: {
		id: 'app.skins.catalog.preview.title',
		defaultMessage: 'Preview',
	},
	selectSkinHint: {
		id: 'app.skins.catalog.preview.select-hint',
		defaultMessage: 'Select a skin',
	},
	armStyleLabel: {
		id: 'app.skins.catalog.preview.arm-style',
		defaultMessage: 'Arm style:',
	},
	classicArm: {
		id: 'app.skins.catalog.arm.classic',
		defaultMessage: 'Classic',
	},
	slimArm: {
		id: 'app.skins.catalog.arm.slim',
		defaultMessage: 'Slim',
	},
	equipButton: {
		id: 'app.skins.catalog.button.equip',
		defaultMessage: 'Equip skin',
	},
	saveToLibraryButton: {
		id: 'app.skins.catalog.button.save-to-library',
		defaultMessage: 'Save to library',
	},
	downloadPngButton: {
		id: 'app.skins.catalog.button.download-png',
		defaultMessage: 'Download PNG',
	},
	catalogErrorTitle: {
		id: 'app.skins.catalog.error.catalog-title',
		defaultMessage: 'Catalog error',
	},
	catalogErrorDefault: {
		id: 'app.skins.catalog.error.catalog-default',
		defaultMessage: 'Failed to load skin catalog',
	},
	playerErrorTitle: {
		id: 'app.skins.catalog.error.player-title',
		defaultMessage: 'Player lookup error',
	},
	playerErrorDefault: {
		id: 'app.skins.catalog.error.player-default',
		defaultMessage: 'Failed to find player skin',
	},
	skinNotSelectedError: {
		id: 'app.skins.catalog.error.not-selected',
		defaultMessage: 'No skin selected',
	},
	equipSuccessTitle: {
		id: 'app.skins.catalog.equip.success-title',
		defaultMessage: 'Skin equipped!',
	},
	equipSuccessText: {
		id: 'app.skins.catalog.equip.success-text',
		defaultMessage: 'Skin "{name}" has been successfully equipped to your account.',
	},
	equipErrorTitle: {
		id: 'app.skins.catalog.equip.error-title',
		defaultMessage: 'Failed to equip skin',
	},
	saveSuccessTitle: {
		id: 'app.skins.catalog.save.success-title',
		defaultMessage: 'Skin saved!',
	},
	saveSuccessText: {
		id: 'app.skins.catalog.save.success-text',
		defaultMessage: 'Skin "{name}" has been added to your saved skins.',
	},
	saveErrorTitle: {
		id: 'app.skins.catalog.save.error-title',
		defaultMessage: 'Failed to save skin',
	},
	downloadErrorTitle: {
		id: 'app.skins.catalog.download.error-title',
		defaultMessage: 'Download error',
	},
	downloadErrorText: {
		id: 'app.skins.catalog.download.error-text',
		defaultMessage: 'Failed to download skin file',
	},
})

type Mode = 'catalog' | 'player'
const activeMode = ref<Mode>('catalog')

// Catalog search state
const catalogQuery = ref('')
const catalogPage = ref(1)
const catalogResult = ref<CatalogSearchResult | null>(null)
const isCatalogLoading = ref(false)

// Player search state
const playerQuery = ref('')
const playerResult = ref<SearchedSkin | null>(null)
const isPlayerLoading = ref(false)
const playerSearched = ref(false)

// Currently selected skin for 3D inspection
const selectedSkin = ref<SearchedSkin | null>(null)
const selectedVariant = ref<SkinModel>('CLASSIC')

// Action loading states
const isEquipping = ref(false)
const isSaving = ref(false)

const popularTagItems = [
	{ key: 'all', label: messages.tagAll, query: '' },
	{ key: 'creeper', label: 'Creeper', query: 'Creeper' },
	{ key: 'anime', label: 'Anime', query: 'Anime' },
	{ key: 'knight', label: 'Knight', query: 'Knight' },
	{ key: 'boy', label: 'Boy', query: 'Boy' },
	{ key: 'girl', label: 'Girl', query: 'Girl' },
	{ key: 'femboy', label: 'Femboy', query: 'Femboy' },
	{ key: 'cool', label: 'Cool', query: 'Cool' },
	{ key: 'pvp', label: 'PvP', query: 'PvP' },
	{ key: 'cute', label: 'Cute', query: 'Cute' },
	{ key: 'steve', label: 'Steve', query: 'Steve' },
	{ key: 'cat', label: 'Cat', query: 'Cat' },
]

const currentPreviewTexture = computed(() => {
	if (!selectedSkin.value) return ''
	if (selectedSkin.value.texture_base64) {
		return selectedSkin.value.texture_base64
	}
	return selectedSkin.value.texture_url
})

function show(initialQuery?: string) {
	modal.value?.show()
	if (initialQuery && initialQuery.trim()) {
		catalogQuery.value = initialQuery.trim()
		void loadCatalog(1, catalogQuery.value)
	} else if (!catalogResult.value) {
		void loadCatalog(1)
	}
}

function hide() {
	modal.value?.hide()
}

defineExpose({
	show,
	hide,
})

async function loadCatalog(page = 1, query = '') {
	isCatalogLoading.value = true
	catalogPage.value = page
	try {
		const res = await search_catalog_skins(query.trim() || undefined, page)
		catalogResult.value = res
		if (res.skins.length > 0 && !selectedSkin.value) {
			selectSkin(res.skins[0])
		}
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.catalogErrorTitle),
			text: error instanceof Error ? error.message : formatMessage(messages.catalogErrorDefault),
		})
	} finally {
		isCatalogLoading.value = false
	}
}

function onSearchCatalog() {
	void loadCatalog(1, catalogQuery.value)
}

function onTagClick(query: string) {
	catalogQuery.value = query
	void loadCatalog(1, query)
}

async function onSearchPlayer() {
	const q = playerQuery.value.trim()
	if (!q) return

	isPlayerLoading.value = true
	playerSearched.value = true
	playerResult.value = null

	try {
		const res = await search_player_skin(q)
		playerResult.value = res
		if (res) {
			selectSkin(res)
		}
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.playerErrorTitle),
			text: error instanceof Error ? error.message : formatMessage(messages.playerErrorDefault),
		})
	} finally {
		isPlayerLoading.value = false
	}
}

function selectSkin(skin: SearchedSkin) {
	selectedSkin.value = skin
	selectedVariant.value = skin.variant
}

async function getSelectedSkinBytes(): Promise<Uint8Array> {
	if (!selectedSkin.value) throw new Error(formatMessage(messages.skinNotSelectedError))

	if (selectedSkin.value.texture_base64 && selectedSkin.value.texture_base64.startsWith('data:image/png;base64,')) {
		const b64 = selectedSkin.value.texture_base64.replace('data:image/png;base64,', '')
		const binary = atob(b64)
		const bytes = new Uint8Array(binary.length)
		for (let i = 0; i < binary.length; i++) {
			bytes[i] = binary.charCodeAt(i)
		}
		return bytes
	}

	return await download_skin_bytes(selectedSkin.value.texture_url)
}

async function equipCurrentSkin() {
	if (!selectedSkin.value) return
	isEquipping.value = true

	try {
		const bytes = await getSelectedSkinBytes()
		const equipped = await add_and_equip_custom_skin(bytes, selectedVariant.value)
		addNotification({
			type: 'success',
			title: formatMessage(messages.equipSuccessTitle),
			text: formatMessage(messages.equipSuccessText, { name: selectedSkin.value.name }),
		})
		emit('equipped', equipped)
		hide()
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.equipErrorTitle),
			text: error instanceof Error ? error.message : String(error),
		})
	} finally {
		isEquipping.value = false
	}
}

async function saveCurrentSkin() {
	if (!selectedSkin.value) return
	isSaving.value = true

	try {
		const bytes = await getSelectedSkinBytes()
		const b64 = arrayBufferToBase64(bytes)
		const dummySkin: Skin = {
			texture_key: selectedSkin.value.texture_key || `custom-${Date.now()}`,
			name: selectedSkin.value.name,
			variant: selectedVariant.value,
			texture: `data:image/png;base64,${b64}`,
			source: 'custom',
			is_equipped: false,
		}

		const saved = await save_custom_skin(dummySkin, bytes, selectedVariant.value, undefined, true)
		addNotification({
			type: 'success',
			title: formatMessage(messages.saveSuccessTitle),
			text: formatMessage(messages.saveSuccessText, { name: selectedSkin.value.name }),
		})
		emit('saved', saved)
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.saveErrorTitle),
			text: error instanceof Error ? error.message : String(error),
		})
	} finally {
		isSaving.value = false
	}
}

async function downloadCurrentSkinFile() {
	if (!selectedSkin.value) return
	try {
		const bytes = await getSelectedSkinBytes()
		const blob = new Blob([bytes], { type: 'image/png' })
		const url = URL.createObjectURL(blob)
		const link = document.createElement('a')
		link.href = url
		link.download = `${selectedSkin.value.name || 'skin'}.png`
		document.body.appendChild(link)
		link.click()
		document.body.removeChild(link)
		URL.revokeObjectURL(url)
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.downloadErrorTitle),
			text: error instanceof Error ? error.message : formatMessage(messages.downloadErrorText),
		})
	}
}
</script>

<template>
	<NewModal ref="modal" class="!max-w-5xl !w-[92vw]">
		<template #title>
			<div class="flex items-center gap-2">
				<GlobeIcon class="size-6 text-brand" />
				<span class="text-xl font-bold text-contrast">{{ formatMessage(messages.catalogModalTitle) }}</span>
			</div>
		</template>

		<div class="flex flex-col gap-4">
			<!-- Mode switchers -->
			<div class="flex items-center justify-between border-b border-surface-5 pb-3">
				<div class="flex gap-2">
					<Button
						:type="activeMode === 'catalog' ? 'colored' : 'base'"
						:color="activeMode === 'catalog' ? 'brand' : undefined"
						size="sm"
						@click="activeMode = 'catalog'"
					>
						<ShirtIcon /> {{ formatMessage(messages.catalogTab) }}
					</Button>
					<Button
						:type="activeMode === 'player' ? 'colored' : 'base'"
						:color="activeMode === 'player' ? 'brand' : undefined"
						size="sm"
						@click="activeMode = 'player'"
					>
						<UserSearchIcon /> {{ formatMessage(messages.playerTab) }}
					</Button>
				</div>
			</div>

			<!-- Main modal body -->
			<div class="grid grid-cols-1 md:grid-cols-[1fr_260px] gap-6 min-h-[460px]">
				<!-- Left Column: Search & Items -->
				<div class="flex flex-col gap-4 min-w-0">
					<!-- CATALOG MODE -->
					<div v-if="activeMode === 'catalog'" class="flex flex-col gap-3">
						<form class="flex gap-2" @submit.prevent="onSearchCatalog">
							<div class="relative flex-1">
								<SearchIcon class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-secondary pointer-events-none" />
								<input
									v-model="catalogQuery"
									type="text"
									:placeholder="formatMessage(messages.catalogSearchPlaceholder)"
									class="w-full bg-surface-3 text-contrast rounded-xl pl-9 pr-8 py-2 text-sm border border-surface-5 outline-none focus:border-brand"
								/>
								<button
									v-if="catalogQuery"
									type="button"
									class="absolute right-2.5 top-1/2 -translate-y-1/2 text-secondary hover:text-contrast"
									@click="catalogQuery = ''; onSearchCatalog()"
								>
									<XIcon class="size-4" />
								</button>
							</div>
							<Button type="colored" color="brand" size="sm" :disabled="isCatalogLoading" @click="onSearchCatalog">
								<SpinnerIcon v-if="isCatalogLoading" class="animate-spin" />
								<span v-else>{{ formatMessage(commonMessages.searchLabel) }}</span>
							</Button>
						</form>

						<!-- Quick tags -->
						<div class="flex flex-wrap gap-1.5">
							<button
								v-for="item in popularTagItems"
								:key="item.key"
								type="button"
								class="px-2.5 py-1 text-xs font-medium rounded-full border transition-colors"
								:class="
									(!item.query && !catalogQuery) || (item.query && catalogQuery.toLowerCase() === item.query.toLowerCase())
										? 'border-brand bg-brand/10 text-brand'
										: 'border-surface-5 bg-surface-3 text-secondary hover:text-contrast'
								"
								@click="onTagClick(item.query)"
							>
								{{ typeof item.label === 'string' ? item.label : formatMessage(item.label) }}
							</button>
						</div>

						<!-- Catalog skin grid -->
						<div class="relative flex-1 min-h-[320px]">
							<div v-if="isCatalogLoading" class="flex h-64 items-center justify-center">
								<SpinnerIcon class="size-8 animate-spin text-brand" />
							</div>

							<div
								v-else-if="catalogResult?.skins && catalogResult.skins.length > 0"
								class="grid grid-cols-3 sm:grid-cols-4 md:grid-cols-5 gap-3 max-h-[360px] overflow-y-auto pr-1"
							>
								<div
									v-for="skin in catalogResult.skins"
									:key="skin.id"
									class="group relative flex flex-col items-center p-2 rounded-xl border transition-all cursor-pointer bg-surface-3 hover:border-brand"
									:class="selectedSkin?.id === skin.id ? 'border-brand ring-2 ring-brand/30' : 'border-surface-5'"
									@click="selectSkin(skin)"
								>
									<!-- Skin Head 2D Face thumbnail -->
									<div class="relative size-14 mb-1.5 rounded-lg overflow-hidden bg-surface-4 flex items-center justify-center">
										<img
											:src="skin.texture_url"
											alt=""
											class="absolute size-56 object-none pointer-events-none"
											style="image-rendering: pixelated; transform: translate(14%, 14%) scale(3.5); transform-origin: 0 0;"
										/>
									</div>
									<span class="text-xs font-semibold text-contrast truncate w-full text-center">
										{{ skin.name }}
									</span>
									<span class="text-[10px] text-secondary">
										{{ skin.variant === 'SLIM' ? formatMessage(messages.slimArm) : formatMessage(messages.classicArm) }}
									</span>
								</div>
							</div>

							<div v-else class="flex h-64 flex-col items-center justify-center text-secondary gap-2">
								<SearchIcon class="size-8" />
								<span>{{ formatMessage(messages.noSkinsFound) }}</span>
							</div>
						</div>

						<!-- Pagination -->
						<div
							v-if="catalogResult && catalogResult.total_pages > 1"
							class="flex items-center justify-between border-t border-surface-5 pt-3 text-xs text-secondary"
						>
							<Button
								size="sm"
								:disabled="catalogPage <= 1 || isCatalogLoading"
								@click="loadCatalog(catalogPage - 1, catalogQuery)"
							>
								{{ formatMessage(commonMessages.backButton) }}
							</Button>
							<span>{{ formatMessage(messages.pageInfo, { page: catalogPage, totalPages: catalogResult.total_pages }) }}</span>
							<Button
								size="sm"
								:disabled="catalogPage >= catalogResult.total_pages || isCatalogLoading"
								@click="loadCatalog(catalogPage + 1, catalogQuery)"
							>
								{{ formatMessage(commonMessages.nextButton) }}
							</Button>
						</div>
					</div>

					<!-- PLAYER LOOKUP MODE -->
					<div v-else class="flex flex-col gap-4">
						<form class="flex gap-2" @submit.prevent="onSearchPlayer">
							<div class="relative flex-1">
								<UserSearchIcon class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-secondary pointer-events-none" />
								<input
									v-model="playerQuery"
									type="text"
									:placeholder="formatMessage(messages.playerSearchPlaceholder)"
									class="w-full bg-surface-3 text-contrast rounded-xl pl-9 pr-8 py-2 text-sm border border-surface-5 outline-none focus:border-brand"
								/>
							</div>
							<Button type="colored" color="brand" size="sm" :disabled="isPlayerLoading || !playerQuery.trim()" @click="onSearchPlayer">
								<SpinnerIcon v-if="isPlayerLoading" class="animate-spin" />
								<span v-else>{{ formatMessage(messages.findSkinButton) }}</span>
							</Button>
						</form>

						<div class="flex-1 flex flex-col items-center justify-center min-h-[300px]">
							<div v-if="isPlayerLoading" class="flex flex-col items-center gap-2">
								<SpinnerIcon class="size-8 animate-spin text-brand" />
								<span class="text-xs text-secondary">{{ formatMessage(messages.playerSearchingNotice) }}</span>
							</div>

							<div v-else-if="playerResult" class="flex flex-col items-center gap-3 p-4 rounded-2xl bg-surface-3 border border-surface-5 w-full max-w-sm">
								<div class="flex items-center gap-3 w-full">
									<div class="size-12 rounded-xl bg-surface-4 overflow-hidden relative flex items-center justify-center border border-surface-5">
										<img
											:src="playerResult.texture_base64"
											alt=""
											class="absolute size-48 object-none pointer-events-none"
											style="image-rendering: pixelated; transform: translate(14%, 14%) scale(3.5); transform-origin: 0 0;"
										/>
									</div>
									<div class="flex flex-col min-w-0">
										<span class="text-base font-bold text-contrast truncate">{{ playerResult.name }}</span>
										<span class="text-xs text-secondary font-mono truncate">UUID: {{ playerResult.id }}</span>
										<span class="text-xs text-brand font-medium">{{ formatMessage(messages.modelLabel, { model: playerResult.variant === 'SLIM' ? formatMessage(messages.slimArm) : formatMessage(messages.classicArm) }) }}</span>
									</div>
								</div>
								<div class="text-xs text-secondary text-center w-full">
									{{ formatMessage(messages.playerSkinFoundNotice) }}
								</div>
							</div>

							<div v-else-if="playerSearched" class="flex flex-col items-center gap-2 text-secondary">
								<XIcon class="size-8 text-red" />
								<span class="text-sm">{{ formatMessage(messages.playerNotFound) }}</span>
							</div>

							<div v-else class="flex flex-col items-center gap-2 text-secondary text-center max-w-xs">
								<UserSearchIcon class="size-10 text-secondary/50" />
								<span class="text-sm">{{ formatMessage(messages.playerSearchHint) }}</span>
							</div>
						</div>
					</div>
				</div>

				<!-- Right Column: 3D Preview & Actions -->
				<div class="flex flex-col items-center justify-between p-4 rounded-2xl bg-surface-3 border border-surface-5 min-h-[400px]">
					<div class="w-full flex flex-col items-center gap-2">
						<span class="text-sm font-bold text-contrast truncate max-w-full">
							{{ selectedSkin ? selectedSkin.name : formatMessage(messages.previewTitle) }}
						</span>

						<!-- 3D Model Renderer -->
						<div class="size-56 relative flex items-center justify-center">
							<SkinPreviewRenderer
								v-if="currentPreviewTexture"
								:texture-src="currentPreviewTexture"
								:variant="selectedVariant"
								framing="modal"
								:initial-rotation="Math.PI / 8"
								class="h-full w-full"
							/>
							<div v-else class="flex flex-col items-center justify-center text-secondary gap-1">
								<ShirtIcon class="size-8" />
								<span class="text-xs">{{ formatMessage(messages.selectSkinHint) }}</span>
							</div>
						</div>

						<!-- Arm style selector -->
						<div v-if="selectedSkin" class="w-full flex flex-col gap-1 items-center">
							<span class="text-xs text-secondary">{{ formatMessage(messages.armStyleLabel) }}</span>
							<RadioButtons v-model="selectedVariant" :items="['CLASSIC', 'SLIM']" class="!flex-row text-xs">
								<template #default="{ item }">
									{{ item === 'CLASSIC' ? formatMessage(messages.classicArm) : formatMessage(messages.slimArm) }}
								</template>
							</RadioButtons>
						</div>
					</div>

					<!-- Action Buttons -->
					<div v-if="selectedSkin" class="flex flex-col gap-2 w-full pt-3 border-t border-surface-5">
						<Button
							type="colored"
							color="brand"
							class="w-full justify-center"
							:disabled="isEquipping"
							@click="equipCurrentSkin"
						>
							<SpinnerIcon v-if="isEquipping" class="animate-spin" />
							<CheckIcon v-else />
							{{ formatMessage(messages.equipButton) }}
						</Button>

						<Button
							type="base"
							class="w-full justify-center"
							:disabled="isSaving"
							@click="saveCurrentSkin"
						>
							<SpinnerIcon v-if="isSaving" class="animate-spin" />
							<SaveIcon v-else />
							{{ formatMessage(messages.saveToLibraryButton) }}
						</Button>

						<Button
							type="outlined"
							size="sm"
							class="w-full justify-center"
							@click="downloadCurrentSkinFile"
						>
							<DownloadIcon />
							{{ formatMessage(messages.downloadPngButton) }}
						</Button>
					</div>
				</div>
			</div>
		</div>

		<template #actions>
			<Button type="outlined" @click="hide">
				<XIcon /> {{ formatMessage(commonMessages.closeButton) }}
			</Button>
		</template>
	</NewModal>
</template>
