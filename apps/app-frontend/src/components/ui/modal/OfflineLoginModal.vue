<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="460px">
		<div class="flex flex-col gap-3">
			<p class="m-0 text-secondary text-sm">
				{{ formatMessage(messages.description) }}
			</p>
			<div class="flex flex-col gap-1.5">
				<label class="text-xs font-semibold text-secondary" for="offline-username-input">
					{{ formatMessage(messages.usernameLabel) }}
				</label>
				<Input
					id="offline-username-input"
					v-model="username"
					:placeholder="formatMessage(messages.usernamePlaceholder)"
					maxlength="16"
					autofocus
					@keydown.enter="submit"
				/>
				<span v-if="errorMessage" class="text-xs text-red mt-1">
					{{ errorMessage }}
				</span>
			</div>
		</div>

		<template #actions>
			<div class="flex gap-2 justify-end">
				<Button type="outlined" @click="hide">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="brand" :disabled="loading || !username.trim()" @click="submit">
					<SpinnerIcon v-if="loading" class="animate-spin" />
					<UserIcon v-else />
					{{ formatMessage(messages.submitButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { SpinnerIcon, UserIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	Input,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'

import { login_offline } from '@/helpers/auth.js'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'minecraft-account.offline-login.header',
		defaultMessage: 'Offline Sign In',
	},
	description: {
		id: 'minecraft-account.offline-login.description',
		defaultMessage:
			'Enter a username to play without a Microsoft account. You can play singleplayer, on offline servers, and configure skins.',
	},
	usernameLabel: {
		id: 'minecraft-account.offline-login.username-label',
		defaultMessage: 'Player username',
	},
	usernamePlaceholder: {
		id: 'minecraft-account.offline-login.username-placeholder',
		defaultMessage: 'For example, Steve',
	},
	submitButton: {
		id: 'minecraft-account.offline-login.submit',
		defaultMessage: 'Sign in',
	},
	errorUsernameRequired: {
		id: 'minecraft-account.offline-login.error.required',
		defaultMessage: 'Please enter a username',
	},
	errorUsernameLength: {
		id: 'minecraft-account.offline-login.error.length',
		defaultMessage: 'Username must be between 3 and 16 characters',
	},
	errorUsernameCharacters: {
		id: 'minecraft-account.offline-login.error.characters',
		defaultMessage: 'Username can only contain Latin letters, numbers, and underscores',
	},
})

const emit = defineEmits<{
	(e: 'logged-in', account: any): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const username = ref('')
const loading = ref(false)
const errorMessage = ref('')

function show() {
	username.value = ''
	errorMessage.value = ''
	loading.value = false
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

async function submit() {
	const trimmed = username.value.trim()
	if (!trimmed) {
		errorMessage.value = formatMessage(messages.errorUsernameRequired)
		return
	}
	if (trimmed.length < 3 || trimmed.length > 16) {
		errorMessage.value = formatMessage(messages.errorUsernameLength)
		return
	}
	if (!/^[a-zA-Z0-9_]+$/.test(trimmed)) {
		errorMessage.value = formatMessage(messages.errorUsernameCharacters)
		return
	}

	errorMessage.value = ''
	loading.value = true
	try {
		const account = await login_offline(trimmed)
		emit('logged-in', account)
		hide()
	} catch (err: any) {
		errorMessage.value = err?.message ?? String(err)
	} finally {
		loading.value = false
	}
}

defineExpose({
	show,
	hide,
})
</script>
