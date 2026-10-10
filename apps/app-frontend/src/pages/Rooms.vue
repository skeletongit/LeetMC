<script setup lang="ts">
import {
	DownloadIcon,
	GameIcon,
	PlayIcon,
	PlusIcon,
	RightArrowIcon,
	SettingsIcon,
	SpinnerIcon,
	UnlinkIcon,
	UploadIcon,
	UsersIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	ConfirmModal,
	defineMessages,
	EmptyState,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref } from 'vue'
import { useRouter } from 'vue-router'

import RoomManagementModal from '@/components/ui/rooms/RoomManagementModal.vue'
import SharedInstancePublishModal from '@/components/ui/shared-instances/SharedInstancePublishModal.vue'
import SharedInstanceUpdateModal from '@/components/ui/shared-instances/SharedInstanceUpdateModal.vue'
import { handleSevereError } from '@/composables/use-error.js'
import {
	getSharedInstanceUnavailableReason,
	install_get_shared_instance_update_preview,
	isSharedInstanceUnavailableError,
} from '@/helpers/install'
import { edit, getInstanceIconUrl, invite_shared_instance_users, run } from '@/helpers/instance'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'
import type { GameInstance, SharedInstanceAttachment } from '@/helpers/types'
import { instanceKeys, instanceListQueryOptions } from '@/pages/instance/query-options'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'

const { formatMessage } = useVIntl()
const { notifySharedInstanceError, notifySharedInstanceUnavailable } = useSharedInstanceErrors()
const router = useRouter()
const queryClient = useQueryClient()
const createRoomModal = ref<InstanceType<typeof NewModal>>()
const unlinkModpackModal = ref<InstanceType<typeof ConfirmModal>>()
const publishModal = ref<InstanceType<typeof SharedInstancePublishModal>>()
const updateModal = ref<InstanceType<typeof SharedInstanceUpdateModal>>()
const roomManagementModal = ref<InstanceType<typeof RoomManagementModal>>()
const selectedPublishInstance = ref<GameInstance | null>(null)
const selectedManageInstance = ref<GameInstance | null>(null)
const pendingUnlinkInstance = ref<GameInstance | null>(null)
const creatingInstanceId = ref<string | null>(null)
const checkingInstanceId = ref<string | null>(null)
const launchingInstanceId = ref<string | null>(null)

const messages = defineMessages({
	title: { id: 'app.rooms.title', defaultMessage: 'Rooms' },
	createRoom: { id: 'app.rooms.create', defaultMessage: 'Create room' },
	createRoomDescription: {
		id: 'app.rooms.create.description',
		defaultMessage: 'Choose the instance you want to share, then invite your friends.',
	},
	noRooms: { id: 'app.rooms.empty.title', defaultMessage: 'No rooms yet' },
	noRoomsDescription: {
		id: 'app.rooms.empty.description',
		defaultMessage:
			'Create a room from one of your instances, or accept an invitation from a friend.',
	},
	noAvailableInstances: {
		id: 'app.rooms.create.no-instances',
		defaultMessage: 'Every available instance already has a room.',
	},
	unlinkRequiredTitle: {
		id: 'app.rooms.create.unlink-required.title',
		defaultMessage: 'Unlink modpack to create room?',
	},
	unlinkRequiredDescription: {
		id: 'app.rooms.create.unlink-required.description',
		defaultMessage:
			'{name} was imported from an .mrpack file. LeetMC must unlink it from the original modpack before creating a room. This will not delete any mods, settings, or worlds, but this instance will no longer receive modpack updates.',
	},
	unlinkAndCreate: {
		id: 'app.rooms.create.unlink-required.proceed',
		defaultMessage: 'Unlink and create room',
	},
	owner: { id: 'app.rooms.role.owner', defaultMessage: 'Owner' },
	member: { id: 'app.rooms.role.member', defaultMessage: 'Member' },
	upToDate: { id: 'app.rooms.status.up-to-date', defaultMessage: 'Synced' },
	changesNotSent: {
		id: 'app.rooms.status.changes-not-sent',
		defaultMessage: 'Not synced',
	},
	updateAvailable: {
		id: 'app.rooms.status.update-available',
		defaultMessage: 'Update available',
	},
	applying: { id: 'app.rooms.status.applying', defaultMessage: 'Applying update' },
	statusUnknown: { id: 'app.rooms.status.unknown', defaultMessage: 'Status unavailable' },
	play: { id: 'app.rooms.action.play', defaultMessage: 'Play' },
	acceptChanges: { id: 'app.rooms.action.accept-changes', defaultMessage: 'Accept changes' },
	sendChanges: { id: 'app.rooms.action.send-changes', defaultMessage: 'Sync changes' },
	manage: { id: 'app.rooms.action.manage', defaultMessage: 'Manage' },
	openInstance: { id: 'app.rooms.action.open-instance', defaultMessage: 'Open instance' },
})

useRootBreadcrumb({
	slot: 'root',
	id: 'rooms',
	label: formatMessage(messages.title),
	to: '/rooms',
	visual: { type: 'icon', component: UsersIcon },
})

const instancesQuery = useQuery(instanceListQueryOptions())
const instances = computed(() => instancesQuery.data.value ?? [])
const rooms = computed(() => instances.value.filter((instance) => !!instance.shared_instance))
const availableInstances = computed(() =>
	instances.value.filter((instance) => !instance.shared_instance && !instance.quarantined),
)

function statusMessage(attachment: SharedInstanceAttachment) {
	if (attachment.status === 'stale') return messages.changesNotSent
	if (attachment.status === 'update_available') return messages.updateAvailable
	if (attachment.status === 'applying') return messages.applying
	if (attachment.status === 'up_to_date') return messages.upToDate
	return messages.statusUnknown
}

function statusClass(attachment: SharedInstanceAttachment) {
	if (attachment.status === 'stale') return 'text-orange'
	if (attachment.status === 'update_available') return 'text-blue'
	if (attachment.status === 'up_to_date') return 'text-green'
	return 'text-secondary'
}

function openCreateRoom() {
	createRoomModal.value?.show()
}

async function createRoom(instance: GameInstance, unlinkImportedModpack = false) {
	if (creatingInstanceId.value) return
	if (instance.link?.type === 'imported_modpack' && !unlinkImportedModpack) {
		pendingUnlinkInstance.value = instance
		createRoomModal.value?.hide()
		await nextTick()
		unlinkModpackModal.value?.show()
		return
	}

	creatingInstanceId.value = instance.id
	createRoomModal.value?.hide()
	try {
		if (unlinkImportedModpack) {
			await edit(instance.id, {
				link: null as unknown as undefined,
			})
		}
		await invite_shared_instance_users(instance.id, [])
		await refreshRooms()
	} catch (error) {
		notifySharedInstanceError(error)
	} finally {
		creatingInstanceId.value = null
	}
}

async function unlinkAndCreateRoom() {
	const instance = pendingUnlinkInstance.value
	if (!instance) return
	pendingUnlinkInstance.value = null
	await createRoom(instance, true)
}

async function openInstance(instance: GameInstance) {
	await router.push(`/instance/${encodeURIComponent(instance.id)}`)
}

async function manageRoom(instance: GameInstance, event?: MouseEvent) {
	selectedManageInstance.value = instance
	await nextTick()
	roomManagementModal.value?.show(event)
}

async function refreshRooms() {
	await queryClient.invalidateQueries({ queryKey: instanceKeys.list() })
}

async function reviewChanges(instance: GameInstance, event?: MouseEvent) {
	selectedPublishInstance.value = instance
	await nextTick()
	publishModal.value?.show(event)
}

async function getRoomUpdate(instance: GameInstance) {
	if (instance.shared_instance?.role !== 'member') return null

	checkingInstanceId.value = instance.id
	try {
		return await install_get_shared_instance_update_preview(instance.id)
	} catch (error) {
		if (isSharedInstanceUnavailableError(error)) {
			notifySharedInstanceUnavailable(getSharedInstanceUnavailableReason(error))
			return undefined
		}
		notifySharedInstanceError(error)
		return undefined
	} finally {
		checkingInstanceId.value = null
	}
}

async function acceptChanges(instance: GameInstance, event?: MouseEvent) {
	const preview = await getRoomUpdate(instance)
	if (preview === undefined) return
	if (!preview?.updateAvailable) {
		await refreshRooms()
		return
	}

	updateModal.value?.show(instance, preview, refreshRooms, event)
}

async function launch(instance: GameInstance) {
	if (checkingInstanceId.value || launchingInstanceId.value || instance.quarantined) return

	const preview = await getRoomUpdate(instance)
	if (instance.shared_instance?.role === 'member' && preview === undefined) return
	if (preview?.updateAvailable) {
		updateModal.value?.show(instance, preview, async () => {
			await refreshRooms()
			await launchGame(instance)
		})
		return
	}

	await launchGame(instance)
}

async function launchGame(instance: GameInstance) {
	launchingInstanceId.value = instance.id
	try {
		await run(instance.id)
	} catch (error) {
		handleSevereError(error, { instanceId: instance.id })
	} finally {
		launchingInstanceId.value = null
	}
}

function handleSharedInstanceUnavailable(
	reason: Parameters<typeof notifySharedInstanceUnavailable>[0],
) {
	notifySharedInstanceUnavailable(reason)
}
</script>

<template>
	<div
		class="box-border flex h-full w-full flex-col gap-6 p-6"
		:class="{ 'justify-center': rooms.length === 0 }"
	>
		<div v-if="rooms.length" class="flex justify-start">
			<Button v-if="rooms.length" type="colored" color="brand" size="lg" @click="openCreateRoom">
				<PlusIcon />
				{{ formatMessage(messages.createRoom) }}
			</Button>
		</div>

		<div v-if="rooms.length" class="grid grid-cols-1 gap-4 xl:grid-cols-2">
			<div
				v-for="instance in rooms"
				:key="instance.id"
				class="flex min-w-0 flex-col gap-4 rounded-2xl border border-solid border-surface-5 bg-bg-raised p-5"
			>
				<div class="flex min-w-0 items-center gap-3">
					<Avatar
						size="56px"
						:src="getInstanceIconUrl(instance.icon_path)"
						:tint-by="instance.id"
						:alt="instance.name"
						pad-transparent-corners
					/>
					<div class="flex min-w-0 flex-1 flex-col gap-1">
						<div class="flex min-w-0 flex-wrap items-center gap-2">
							<h2 class="m-0 truncate text-xl text-contrast">{{ instance.name }}</h2>
							<span
								class="rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1 text-xs font-semibold text-secondary"
							>
								{{
									formatMessage(
										instance.shared_instance?.role === 'owner' ? messages.owner : messages.member,
									)
								}}
							</span>
							<span
								v-if="instance.shared_instance"
								class="inline-flex items-center gap-1.5 text-xs font-semibold"
								:class="statusClass(instance.shared_instance)"
							>
								{{ formatMessage(statusMessage(instance.shared_instance)) }}
							</span>
						</div>
						<div class="flex items-center gap-1.5 text-sm font-semibold text-secondary">
							<GameIcon class="size-4" />
							<span class="capitalize">{{ instance.loader }} {{ instance.game_version }}</span>
						</div>
					</div>
				</div>

				<div class="flex flex-wrap items-center gap-2">
					<Button
						type="colored"
						color="brand"
						:disabled="
							!!checkingInstanceId ||
							!!launchingInstanceId ||
							instance.install_stage !== 'installed'
						"
						@click="launch(instance)"
					>
						<SpinnerIcon
							v-if="checkingInstanceId === instance.id || launchingInstanceId === instance.id"
							class="animate-spin"
						/>
						<PlayIcon v-else />
						{{ formatMessage(messages.play) }}
					</Button>
					<Button
						v-if="
							instance.shared_instance?.role === 'member' &&
							instance.shared_instance.status === 'update_available'
						"
						type="outlined"
						:disabled="!!checkingInstanceId"
						@click="acceptChanges(instance, $event)"
					>
						<SpinnerIcon v-if="checkingInstanceId === instance.id" class="animate-spin" />
						<DownloadIcon v-else />
						{{ formatMessage(messages.acceptChanges) }}
					</Button>
					<Button
						v-if="instance.shared_instance?.role === 'owner'"
						type="outlined"
						@click="reviewChanges(instance, $event)"
					>
						<UploadIcon />
						{{ formatMessage(messages.sendChanges) }}
					</Button>
					<Button
						v-if="instance.shared_instance?.role === 'owner'"
						type="outlined"
						@click="manageRoom(instance, $event)"
					>
						<SettingsIcon />
						{{ formatMessage(messages.manage) }}
					</Button>
					<Button v-else type="quiet" @click="openInstance(instance)">
						<RightArrowIcon />
						{{ formatMessage(messages.openInstance) }}
					</Button>
				</div>
			</div>
		</div>

		<EmptyState
			v-else
			type="empty-inbox"
			:heading="formatMessage(messages.noRooms)"
			:description="formatMessage(messages.noRoomsDescription)"
		>
			<template #actions>
				<Button type="colored" color="brand" size="lg" @click="openCreateRoom">
					<PlusIcon />
					{{ formatMessage(messages.createRoom) }}
				</Button>
			</template>
		</EmptyState>
	</div>

	<NewModal
		ref="createRoomModal"
		:header="formatMessage(messages.createRoom)"
		width="560px"
		max-width="560px"
		scrollable
	>
		<div class="flex flex-col gap-4 pb-5">
			<p class="m-0 text-secondary">{{ formatMessage(messages.createRoomDescription) }}</p>
			<div v-if="availableInstances.length" class="flex flex-col gap-2">
				<button
					v-for="instance in availableInstances"
					:key="instance.id"
					class="button-base flex w-full cursor-pointer items-center gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 p-3 text-left"
					:disabled="!!creatingInstanceId"
					@click="createRoom(instance)"
				>
					<Avatar
						size="44px"
						:src="getInstanceIconUrl(instance.icon_path)"
						:tint-by="instance.id"
						:alt="instance.name"
						pad-transparent-corners
					/>
					<div class="flex min-w-0 flex-1 flex-col">
						<span class="truncate font-semibold text-contrast">{{ instance.name }}</span>
						<span class="text-sm capitalize text-secondary">
							{{ instance.loader }} {{ instance.game_version }}
						</span>
					</div>
					<SpinnerIcon v-if="creatingInstanceId === instance.id" class="shrink-0 animate-spin" />
					<RightArrowIcon v-else class="shrink-0" />
				</button>
			</div>
			<div v-else class="rounded-xl bg-surface-2 p-4 text-secondary">
				{{ formatMessage(messages.noAvailableInstances) }}
			</div>
		</div>
	</NewModal>
	<ConfirmModal
		ref="unlinkModpackModal"
		:title="formatMessage(messages.unlinkRequiredTitle)"
		:description="
			formatMessage(messages.unlinkRequiredDescription, {
				name: pendingUnlinkInstance?.name ?? '',
			})
		"
		:proceed-label="formatMessage(messages.unlinkAndCreate)"
		:proceed-icon="UnlinkIcon"
		:danger="false"
		:markdown="false"
		@proceed="unlinkAndCreateRoom"
	/>

	<SharedInstancePublishModal
		v-if="selectedPublishInstance"
		ref="publishModal"
		:key="selectedPublishInstance.id"
		:instance="selectedPublishInstance"
		@published="refreshRooms"
	/>
	<RoomManagementModal
		v-if="selectedManageInstance"
		ref="roomManagementModal"
		:key="selectedManageInstance.id"
		:instance="selectedManageInstance"
		@updated="refreshRooms"
	/>
	<SharedInstanceUpdateModal
		ref="updateModal"
		@shared-instance-unavailable="handleSharedInstanceUnavailable"
	/>
</template>
