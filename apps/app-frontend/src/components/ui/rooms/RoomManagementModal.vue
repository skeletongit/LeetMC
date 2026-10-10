<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header, { name: instance.name })"
		width="800px"
		max-width="800px"
		scrollable
	>
		<ModrinthAccountRequiredModal ref="accountRequiredModal" :request-auth="requestAuth" />
		<InvitePlayersModal
			ref="invitePlayersModal"
			:header="formatMessage(messages.inviteHeader, { name: instance.name })"
			:friends="inviteFriends"
			:search-users="searchInviteUsers"
			:link="inviteLink.link.value"
			:link-expires-at="inviteLink.details.value?.expiresAt"
			:link-max-uses="inviteLink.details.value?.maxUses"
			:link-max-uses-limit="remainingUserSlots"
			:update-invite-link="inviteLink.update"
			:user-profile-link="userProfileLink"
			:can-invite="hasRemainingUserSlots && !members.exclusiveMutationPending.value"
			:invite-disabled-message="
				hasRemainingUserSlots
					? undefined
					: formatMessage(messages.userLimitReached, { limit: SHARED_INSTANCE_USER_LIMIT })
			"
			@invite="invitePlayer"
			@cancel="cancelInvite"
		/>
		<SharedInstanceRemoveMemberModal
			ref="removeMemberModal"
			:row="pendingRemovalRow"
			@confirm="removeMember"
			@clear="pendingRemovalRow = null"
		/>
		<ConfirmModal
			ref="unlinkRoomModal"
			:title="formatMessage(messages.unlinkTitle)"
			:description="formatMessage(messages.unlinkDescription)"
			:proceed-label="formatMessage(messages.unlinkButton)"
			:proceed-icon="UnlinkIcon"
			:markdown="false"
			@proceed="unlinkRoom"
		/>
		<SharedInstancePublishModal
			ref="publishModal"
			:instance="instance"
			@published="handlePublished"
			@state-change="publishState = $event"
		/>

		<div class="flex flex-col gap-5 pb-5">
			<SharedInstanceMembersTable />
			<div class="border-0 border-t border-solid border-surface-5 pt-5">
				<Button
					type="colored"
					color="red"
					:disabled="unlinkingRoom"
					@click="unlinkRoomModal?.show()"
				>
					<SpinnerIcon v-if="unlinkingRoom" class="animate-spin" />
					<UnlinkIcon v-else />
					{{ formatMessage(messages.unlinkButton) }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { SpinnerIcon, UnlinkIcon } from '@modrinth/assets'
import {
	Button,
	ConfirmModal,
	defineMessages,
	injectAuth,
	type InvitePlayersInvitePayload,
	InvitePlayersModal,
	type InvitePlayersUser,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref, toRef } from 'vue'

import ModrinthAccountRequiredModal from '@/components/ui/modal/ModrinthAccountRequiredModal.vue'
import SharedInstancePublishModal from '@/components/ui/shared-instances/SharedInstancePublishModal.vue'
import { unpublish_shared_instance } from '@/helpers/instance'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'
import type { GameInstance } from '@/helpers/types'
import type { ModrinthAuthFlow } from '@/helpers/mr_auth.ts'
import { instanceKeys } from '@/pages/instance/query-options'
import { provideSharedInstanceManagement } from '@/pages/instance/share/shared-instance-management-context'
import SharedInstanceMembersTable from '@/pages/instance/share/shared-instance-members-table.vue'
import SharedInstanceRemoveMemberModal from '@/pages/instance/share/shared-instance-remove-member-modal.vue'
import {
	SHARED_INSTANCE_USER_LIMIT,
	type ShareRow,
} from '@/pages/instance/share/shared-instance-share-types'
import { useSharedInstanceInviteCandidates } from '@/pages/instance/share/use-shared-instance-invite-candidates'
import { useSharedInstanceInviteLink } from '@/pages/instance/share/use-shared-instance-invite-link'
import { useSharedInstanceMembers } from '@/pages/instance/share/use-shared-instance-members'

const props = defineProps<{ instance: GameInstance }>()
const emit = defineEmits<{ updated: [] }>()
const instance = toRef(props, 'instance')
const auth = injectAuth()
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()
const { notifySharedInstanceError } = useSharedInstanceErrors()
const modal = ref<InstanceType<typeof NewModal>>()
const accountRequiredModal = ref<InstanceType<typeof ModrinthAccountRequiredModal>>()
const invitePlayersModal = ref<InstanceType<typeof InvitePlayersModal>>()
const removeMemberModal = ref<InstanceType<typeof SharedInstanceRemoveMemberModal>>()
const publishModal = ref<InstanceType<typeof SharedInstancePublishModal>>()
const unlinkRoomModal = ref<InstanceType<typeof ConfirmModal>>()
const pendingRemovalRow = ref<ShareRow | null>(null)
const publishState = ref<'idle' | 'reviewing' | 'publishing'>('idle')
const unlinkingRoom = ref(false)
const currentUserId = computed(() => auth.user.value?.id ?? null)
const isSignedIn = computed(() => !!auth.session_token.value)
const actionsLocked = computed(
	() =>
		!!instance.value.shared_instance?.linked_user_id &&
		isSignedIn.value &&
		instance.value.shared_instance.linked_user_id !== currentUserId.value,
)

const members = useSharedInstanceMembers({
	instance,
	currentUserId,
	isSignedIn,
	actionsLocked,
	onError: notifySharedInstanceError,
})
const remainingUserSlots = computed(() =>
	Math.max(0, SHARED_INSTANCE_USER_LIMIT - members.rows.value.length),
)
const hasRemainingUserSlots = computed(() => remainingUserSlots.value > 0)
const {
	inviteFriends,
	search: searchInviteUsers,
	requestFriend,
} = useSharedInstanceInviteCandidates({
	rows: members.rows,
	currentUserId,
	isSignedIn,
	actionsLocked,
})
const inviteLink = useSharedInstanceInviteLink(
	computed(() => instance.value.id),
	remainingUserSlots,
	notifySharedInstanceError,
)

const messages = defineMessages({
	header: { id: 'app.rooms.manage.header', defaultMessage: 'Manage {name}' },
	inviteHeader: { id: 'app.rooms.manage.invite-header', defaultMessage: 'Invite to {name}' },
	userLimitReached: {
		id: 'app.rooms.manage.user-limit-reached',
		defaultMessage: 'This room has reached the {limit}-user limit.',
	},
	unlinkButton: { id: 'app.rooms.manage.unlink.button', defaultMessage: 'Unlink room' },
	unlinkTitle: { id: 'app.rooms.manage.unlink.title', defaultMessage: 'Unlink this room?' },
	unlinkDescription: {
		id: 'app.rooms.manage.unlink.description',
		defaultMessage:
			'This will close the room for everyone and stop future synchronization. The instance and all of its installed files will remain on this device.',
	},
})

function show(event?: MouseEvent) {
	modal.value?.show(event)
}

async function showInvitePlayers(event?: MouseEvent) {
	if (actionsLocked.value || !hasRemainingUserSlots.value) return
	if (!isSignedIn.value) {
		await accountRequiredModal.value?.show(event)
		return
	}
	if (await inviteLink.ensure()) invitePlayersModal.value?.show(event)
}

function invitePlayer(payload: InvitePlayersInvitePayload) {
	if (payload.source === 'search') void requestFriend(payload.user)
	members.invite(payload.user)
	invitePlayersModal.value?.hide()
}

function cancelInvite(user: InvitePlayersUser) {
	const row = members.find(user.id, user.username)
	if (row) members.remove(row.id)
}

function showRemoveMemberModal(row: ShareRow) {
	pendingRemovalRow.value = row
	removeMemberModal.value?.show()
}

function removeMember(row: ShareRow) {
	members.remove(row.id)
}

function reviewUpdate(event: MouseEvent) {
	publishModal.value?.show(event)
}

async function handlePublished() {
	await queryClient.invalidateQueries({ queryKey: instanceKeys.list() })
	emit('updated')
}

async function unlinkRoom() {
	if (unlinkingRoom.value) return

	unlinkingRoom.value = true
	try {
		await unpublish_shared_instance(instance.value.id)
		await queryClient.invalidateQueries({ queryKey: instanceKeys.list() })
		emit('updated')
		modal.value?.hide()
	} catch (error) {
		notifySharedInstanceError(error)
	} finally {
		unlinkingRoom.value = false
	}
}

function userProfileLink(username: string) {
	return !username || username.includes('@') ? undefined : `/user/${encodeURIComponent(username)}`
}

async function requestAuth(flow: ModrinthAuthFlow) {
	await auth.requestSignIn('/rooms', flow, { showModal: false })
	return !!auth.session_token.value
}

provideSharedInstanceManagement({
	rows: members.rows,
	actionsLocked,
	inviteDisabled: computed(() => !hasRemainingUserSlots.value),
	invitePending: inviteLink.pending,
	pushUpdateDisabled: computed(
		() => instance.value.install_stage !== 'installed' || publishState.value !== 'idle',
	),
	pushUpdatePending: computed(() => publishState.value !== 'idle'),
	invite: (event) => void showInvitePlayers(event),
	remove: showRemoveMemberModal,
	pushUpdate: reviewUpdate,
})

defineExpose({ show })
</script>
