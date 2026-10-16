<script lang="ts"></script>

<script setup lang="ts">
import { ChevronDownIcon, RightArrowIcon } from '@modrinth/assets'
import { onClickOutside, onKeyStroke, useElementSize } from '@vueuse/core'
import { type Component, type ComponentPublicInstance, computed, nextTick, ref, watch } from 'vue'

import { type MessageDescriptor, useVIntl } from '../../composables/i18n'
import { useScrollIndicator } from '../../composables/scroll-indicator'
import { truncatedTooltip } from '../../utils/truncate'
import LoadingIndicator from '../base/LoadingIndicator.vue'
import NewModal from './NewModal.vue'
export interface Tab {
	name: MessageDescriptor
	category?: MessageDescriptor
	icon: Component
	content?: Component
	href?: string
	badge?: MessageDescriptor
	shown?: boolean
}

defineSlots<{
	title?(): unknown
	'sidebar-header'?(): unknown
	footer?(): unknown
	content?(props: { tab: Tab | undefined; index: number }): unknown
	'floating-action-bar'?(): unknown
}>()

const { formatMessage } = useVIntl()

const props = withDefaults(
	defineProps<{
		tabs: Tab[]
		header?: string
		maxWidth?: string
		width?: string
		closable?: boolean
		onHide?: () => void
		onAfterHide?: () => void
		onShow?: () => void
		beforeHide?: () => boolean
		beforeTabChange?: (fromIndex: number, toIndex: number) => boolean
		hideTabSelection?: boolean
		floatingActionBarShown?: boolean
		disableClose?: boolean
		navPosition?: 'side' | 'top'
	}>(),
	{
		header: undefined,
		maxWidth: undefined,
		width: undefined,
		closable: true,
		onHide: undefined,
		onAfterHide: undefined,
		onShow: undefined,
		beforeHide: undefined,
		beforeTabChange: undefined,
		hideTabSelection: false,
		floatingActionBarShown: false,
		disableClose: false,
		navPosition: 'side',
	},
)

const visibleTabs = computed(() => props.tabs.filter((tab) => tab.shown !== false))

const selectedTab = ref(0)
const tabLabelRefs = ref<Record<number, HTMLElement | null>>({})

function setTabLabelRef(index: number, element: Element | ComponentPublicInstance | null) {
	tabLabelRefs.value[index] = element instanceof HTMLElement ? element : null
}

function tabLabelTooltip(index: number, label: string) {
	return truncatedTooltip(tabLabelRefs.value[index], label)
}

const scrollContainer = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade, checkScrollState, forceCheck } =
	useScrollIndicator(scrollContainer)

const floatingActionBarContainer = ref<HTMLElement | null>(null)
const { height: floatingActionBarHeight } = useElementSize(floatingActionBarContainer)
const contentBottomPadding = computed(() =>
	props.floatingActionBarShown ? `calc(${floatingActionBarHeight.value}px + 2.25rem)` : '1.5rem',
)

watch(contentBottomPadding, () => forceCheck(), { flush: 'post' })

const sidebarScrollContainer = ref<HTMLElement | null>(null)
const {
	showTopFade: showSidebarTopFade,
	showBottomFade: showSidebarBottomFade,
	checkScrollState: checkSidebarScrollState,
} = useScrollIndicator(sidebarScrollContainer)

const modal = ref<InstanceType<typeof NewModal> | null>(null)

function setTab(index: number) {
	if (index === selectedTab.value && !props.hideTabSelection) return
	if (props.beforeTabChange?.(selectedTab.value, index) === false) return
	selectedTab.value = index
	openCategoryDropdown.value = null
	nextTick(() => {
		scrollContainer.value?.scrollTo({ top: 0, behavior: 'instant' })
		forceCheck()
	})
}

interface TabCategoryGroup {
	category: MessageDescriptor | null
	id: string
	tabs: Array<{ tab: Tab; index: number }>
}

const categoryGroups = computed<TabCategoryGroup[]>(() => {
	const groups: TabCategoryGroup[] = []
	const groupMap = new Map<string, TabCategoryGroup>()

	visibleTabs.value.forEach((tab, index) => {
		const category = tab.category
		const categoryId = category ? category.id : '__no_category__'

		let group = groupMap.get(categoryId)
		if (!group) {
			group = {
				category: category ?? null,
				id: categoryId,
				tabs: [],
			}
			groupMap.set(categoryId, group)
			groups.push(group)
		}
		group.tabs.push({ tab, index })
	})

	return groups
})

const activeCategoryGroup = computed(() => {
	const currentTab = visibleTabs.value[selectedTab.value]
	if (!currentTab) return null
	const categoryId = currentTab.category ? currentTab.category.id : '__no_category__'
	return categoryGroups.value.find((g) => g.id === categoryId) ?? null
})

const openCategoryDropdown = ref<string | null>(null)
const topNavContainerRef = ref<HTMLElement | null>(null)

function toggleCategoryDropdown(categoryId: string) {
	if (openCategoryDropdown.value === categoryId) {
		openCategoryDropdown.value = null
	} else {
		openCategoryDropdown.value = categoryId
	}
}

function selectTabFromDropdown(index: number, tab: Tab) {
	if (tab.href) {
		openCategoryDropdown.value = null
		return
	}
	setTab(index)
	openCategoryDropdown.value = null
}

onClickOutside(topNavContainerRef, () => {
	openCategoryDropdown.value = null
})

onKeyStroke('Escape', (event) => {
	if (openCategoryDropdown.value) {
		openCategoryDropdown.value = null
		event.stopPropagation()
	}
})

function show(event?: MouseEvent) {
	modal.value?.show(event)
}

function hide(): boolean {
	return modal.value?.hide() ?? false
}

function startsCategory(index: number) {
	const category = visibleTabs.value[index]?.category
	return !!category && category.id !== visibleTabs.value[index - 1]?.category?.id
}

defineExpose({ show, hide, selectedTab, setTab })
</script>
<template>
	<NewModal
		ref="modal"
		:header="header"
		:max-width="maxWidth"
		:width="width"
		:closable="closable"
		:on-hide="onHide"
		:on-after-hide="onAfterHide"
		:on-show="onShow"
		:before-hide="beforeHide"
		:disable-close="disableClose"
		no-padding
	>
		<template v-if="$slots.title" #title>
			<slot name="title" />
		</template>

		<!-- Top navigation layout -->
		<div v-if="navPosition === 'top'" class="flex flex-col">
			<!-- Top navigation bar with category headers -->
			<div
				ref="topNavContainerRef"
				class="relative z-30 flex flex-wrap items-center gap-2 border-0 border-b border-solid border-divider bg-surface-2/40 px-6 py-3"
			>
				<div
					v-for="group in categoryGroups"
					:key="group.id"
					class="relative"
				>
					<button
						type="button"
						:class="[
							'flex items-center gap-2 rounded-xl px-3.5 py-2 text-sm font-semibold transition-all cursor-pointer border border-solid',
							activeCategoryGroup?.id === group.id
								? 'bg-brand-highlight text-brand border-brand/30 shadow-sm'
								: 'bg-surface-3 text-secondary border-surface-5 hover:bg-surface-4 hover:text-contrast',
							openCategoryDropdown === group.id ? 'ring-2 ring-brand/40' : '',
						]"
						@click="toggleCategoryDropdown(group.id)"
					>
						<span>{{ group.category ? formatMessage(group.category) : 'Settings' }}</span>
						<span
							v-if="activeCategoryGroup?.id === group.id && visibleTabs[selectedTab]"
							class="rounded-lg bg-brand/15 px-2 py-0.5 text-xs font-medium text-brand"
						>
							{{ formatMessage(visibleTabs[selectedTab].name) }}
						</span>
						<ChevronDownIcon
							class="w-4 h-4 shrink-0 transition-transform duration-200"
							:class="{ 'rotate-180': openCategoryDropdown === group.id }"
						/>
					</button>

					<!-- Dropdown menu -->
					<Transition
						enter-active-class="transition duration-150 ease-out"
						enter-from-class="transform scale-95 opacity-0"
						enter-to-class="transform scale-100 opacity-100"
						leave-active-class="transition duration-100 ease-in"
						leave-from-class="transform scale-100 opacity-100"
						leave-to-class="transform scale-95 opacity-0"
					>
						<div
							v-if="openCategoryDropdown === group.id"
							class="absolute left-0 top-full z-50 mt-2 min-w-[220px] rounded-2xl border border-solid border-surface-5 bg-bg-raised p-1.5 shadow-2xl backdrop-blur-md"
						>
							<div class="flex flex-col gap-1">
								<component
									:is="item.tab.href ? 'a' : 'button'"
									v-for="item in group.tabs"
									:key="item.index"
									:href="item.tab.href ?? undefined"
									:target="item.tab.href ? '_blank' : undefined"
									:rel="item.tab.href ? 'noopener noreferrer' : undefined"
									:class="[
										'flex items-center gap-2.5 rounded-xl px-3 py-2 text-left text-sm font-semibold transition-all border-none cursor-pointer',
										!item.tab.href && !hideTabSelection && selectedTab === item.index
											? 'bg-button-bgSelected text-button-textSelected'
											: 'bg-transparent text-secondary hover:bg-surface-3 hover:text-contrast',
									]"
									@click="selectTabFromDropdown(item.index, item.tab)"
								>
									<component :is="item.tab.icon" class="w-4 h-4 shrink-0" />
									<span class="min-w-0 flex-1 truncate">
										{{ formatMessage(item.tab.name) }}
									</span>
									<span
										v-if="item.tab.badge"
										class="shrink-0 rounded-full bg-brand-highlight px-1.5 py-0.5 text-xs font-bold text-brand-green"
									>
										{{ formatMessage(item.tab.badge) }}
									</span>
									<RightArrowIcon v-if="item.tab.href" class="ml-auto size-4 shrink-0" />
								</component>
							</div>
						</div>
					</Transition>
				</div>
			</div>

			<!-- Content area -->
			<div class="relative h-[min(65vh,600px)]">
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-4"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-4"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-4 bg-gradient-to-b from-bg-raised to-transparent"
					/>
				</Transition>

				<div
					ref="scrollContainer"
					class="absolute inset-0 overflow-y-auto px-6 py-6"
					@scroll="checkScrollState"
				>
					<div class="flow-root min-h-full" :style="{ paddingBottom: contentBottomPadding }">
						<slot name="content" :tab="visibleTabs[selectedTab]" :index="selectedTab">
							<Suspense>
								<component
									:is="visibleTabs[selectedTab]?.content"
									v-if="visibleTabs[selectedTab]?.content"
								/>
								<template #fallback>
									<LoadingIndicator class="py-2" />
								</template>
							</Suspense>
						</slot>
					</div>
				</div>

				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-16"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-16"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-16 bg-gradient-to-t from-bg-raised to-transparent"
					/>
				</Transition>

				<div
					ref="floatingActionBarContainer"
					class="pointer-events-none absolute bottom-3 left-6 right-6 z-20"
				>
					<div class="pointer-events-auto">
						<slot name="floating-action-bar" />
					</div>
				</div>
			</div>

			<!-- Footer -->
			<div
				v-if="$slots.footer"
				class="border-0 border-t border-solid border-divider bg-surface-1/40 px-6 py-3"
			>
				<slot name="footer" />
			</div>
		</div>

		<!-- Side layout (default) -->
		<div v-else class="grid grid-cols-[minmax(12.5rem,18rem)_minmax(0,1fr)] p-6 pb-3 pr-0">
			<div
				class="flex min-w-0 max-h-[min(65vh,600px)] flex-col border-0 border-r-[1px] border-solid border-divider pr-4"
			>
				<slot name="sidebar-header" />

				<div class="relative min-h-0 flex-1">
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-4"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-4"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showSidebarTopFade"
							class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-4 bg-gradient-to-b from-bg-raised to-transparent"
						/>
					</Transition>

					<div
						ref="sidebarScrollContainer"
						class="flex h-full flex-col gap-1 overflow-y-auto"
						@scroll="checkSidebarScrollState"
					>
						<template v-for="(tab, index) in visibleTabs" :key="index">
							<div
								v-if="startsCategory(index) && tab.category"
								class="shrink-0 truncate px-4 pb-1 pt-2 text-xs font-bold uppercase tracking-wide text-secondary"
							>
								{{ formatMessage(tab.category) }}
							</div>
							<component
								:is="tab.href ? 'a' : 'button'"
								:href="tab.href ?? undefined"
								:target="tab.href ? '_blank' : undefined"
								:rel="tab.href ? 'noopener noreferrer' : undefined"
								:class="`flex min-w-0 shrink-0 gap-2 items-center text-left rounded-xl px-4 py-2 border-none font-semibold cursor-pointer active:scale-[0.97] transition-all no-underline ${!tab.href && !hideTabSelection && selectedTab === index ? 'bg-button-bgSelected text-button-textSelected' : 'bg-transparent text-button-text hover:bg-button-bg hover:text-contrast'}`"
								@click="!tab.href && setTab(index)"
							>
								<component :is="tab.icon" class="w-4 h-4 flex-shrink-0" />
								<span
									:ref="(element) => setTabLabelRef(index, element)"
									v-tooltip="tabLabelTooltip(index, formatMessage(tab.name))"
									class="min-w-0 flex-1 truncate"
								>
									{{ formatMessage(tab.name) }}
								</span>
								<span
									v-if="tab.badge"
									class="shrink-0 rounded-full px-1.5 py-0.5 text-xs font-bold bg-brand-highlight text-brand-green"
								>
									{{ formatMessage(tab.badge) }}
								</span>
								<RightArrowIcon v-if="tab.href" class="ml-auto size-4 shrink-0" />
							</component>
						</template>
					</div>

					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-16"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-16"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showSidebarBottomFade"
							class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-16 bg-gradient-to-t from-bg-raised to-transparent"
						/>
					</Transition>
				</div>

				<slot name="footer" />
			</div>
			<div class="relative min-h-[min(65vh,600px)]">
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-4"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-4"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-4 bg-gradient-to-b from-bg-raised to-transparent"
					/>
				</Transition>

				<div
					ref="scrollContainer"
					class="absolute inset-0 overflow-y-auto px-6"
					@scroll="checkScrollState"
				>
					<div class="flow-root min-h-full" :style="{ paddingBottom: contentBottomPadding }">
						<slot name="content" :tab="visibleTabs[selectedTab]" :index="selectedTab">
							<Suspense>
								<component
									:is="visibleTabs[selectedTab]?.content"
									v-if="visibleTabs[selectedTab]?.content"
								/>
								<template #fallback>
									<LoadingIndicator class="py-2" />
								</template>
							</Suspense>
						</slot>
					</div>
				</div>

				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-16"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-16"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-16 bg-gradient-to-t from-bg-raised to-transparent"
					/>
				</Transition>

				<div
					ref="floatingActionBarContainer"
					class="pointer-events-none absolute bottom-3 left-6 right-6 z-20"
				>
					<div class="pointer-events-auto">
						<slot name="floating-action-bar" />
					</div>
				</div>
			</div>
		</div>
	</NewModal>
</template>
