<template>
	<div>
		<div
			v-if="accounts.length === 0"
			class="flex flex-col gap-2 bg-button-bg border border-solid border-surface-5 rounded-xl p-3 mt-2"
		>
			<span>{{ formatMessage(messages.notSignedIn) }}</span>
			<Button type="colored" color="brand" :disabled="loginDisabled" @click="login()">
				<LogInIcon v-if="!loginDisabled" />
				<SpinnerIcon v-else class="animate-spin" />
				{{ formatMessage(messages.signInToMinecraft) }}
			</Button>
			<Button
				class="!bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
				:disabled="loginDisabled"
				@click="openOfflineLogin()"
			>
				<UserIcon />
				{{ formatMessage(messages.addOfflineAccount) }}
			</Button>
		</div>
		<Accordion
			v-else
			class="w-full mt-2 bg-button-bg border border-solid border-surface-5 rounded-xl overflow-clip"
			button-class="button-base w-full bg-transparent px-3 py-2 border-0 cursor-pointer"
			:open-by-default="false"
		>
			<template #title>
				<div class="flex gap-2 w-full min-w-0">
					<Avatar
						size="36px"
						:src="
							selectedAccount
								? avatarUrl
								: 'https://launcher-files.modrinth.com/assets/steve_head.png'
						"
					/>
					<div class="flex flex-col items-start w-full min-w-0">
						<span class="truncate w-full text-left">{{
							selectedAccount ? selectedAccount.profile.name : formatMessage(messages.selectAccount)
						}}</span>
						<span class="text-secondary text-xs">
							{{
								selectedAccount?.refresh_token === 'offline'
									? formatMessage(messages.offlineAccountLabel)
									: formatMessage(messages.minecraftAccount)
							}}
						</span>
					</div>
				</div>
			</template>
			<div class="bg-button-bg pt-1 pb-2 border-0 border-t border-solid border-surface-5">
				<template v-if="accounts.length > 0">
					<div
						v-for="account in accounts"
						:key="account.profile.id"
						class="flex gap-1 items-center"
					>
						<button
							class="flex items-center flex-shrink flex-grow overflow-clip gap-2 p-2 border-0 bg-transparent cursor-pointer button-base min-w-0"
							@click="setAccount(account)"
						>
							<RadioButtonCheckedIcon
								v-if="selectedAccount && selectedAccount.profile.id === account.profile.id"
								class="w-5 h-5 text-brand shrink-0"
							/>
							<RadioButtonIcon v-else class="w-5 h-5 text-secondary shrink-0" />
							<Avatar :src="getAccountAvatarUrl(account)" size="24px" />
							<div class="flex flex-col items-start truncate min-w-0">
								<p
									class="m-0 truncate min-w-0"
									:class="
										selectedAccount && selectedAccount.profile.id === account.profile.id
											? 'text-contrast font-semibold'
											: 'text-primary'
									"
								>
									{{ account.profile.name }}
								</p>
								<span
									v-if="account.refresh_token === 'offline'"
									class="text-[10px] text-secondary leading-none uppercase font-semibold"
								>
									{{ formatMessage(messages.offlineBadge) }}
								</span>
							</div>
						</button>
						<IconButton
							v-tooltip="formatMessage(messages.removeAccount)"
							type="quiet"
							color="red"
							:label="formatMessage(messages.removeAccount)"
							class="mr-2 !bg-button-bg !text-primary ![box-shadow:var(--shadow-button)] hover:!bg-red focus-visible:!bg-red hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
							@click="logout(account.profile.id)"
						>
							<TrashIcon />
						</IconButton>
					</div>
				</template>
				<div class="flex flex-col gap-2 px-2 pt-2">
					<Button
						v-if="accounts.length > 0"
						class="w-full !bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
						:disabled="loginDisabled"
						@click="login()"
					>
						<PlusIcon />
						{{ formatMessage(messages.addAccount) }}
					</Button>
					<Button
						v-if="accounts.length > 0"
						class="w-full !bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
						:disabled="loginDisabled"
						@click="openOfflineLogin()"
					>
						<UserIcon />
						{{ formatMessage(messages.addOfflineAccount) }}
					</Button>
				</div>
			</div>
		</Accordion>
		<OfflineLoginModal ref="offlineModal" @logged-in="handleOfflineLoggedIn" />
	</div>
</template>

<script setup lang="ts">
import {
	LogInIcon,
	PlusIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	SpinnerIcon,
	TrashIcon,
	UserIcon,
} from '@modrinth/assets'
import {
	Accordion,
	Avatar,
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import type { Ref } from 'vue'
import { computed, onUnmounted, ref } from 'vue'

import OfflineLoginModal from '@/components/ui/modal/OfflineLoginModal.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import {
	get_default_user,
	login as login_flow,
	remove_user,
	set_default_user,
	users,
} from '@/helpers/auth'
import { getPlayerHeadUrl } from '@/helpers/rendering/player-head'
import type { Skin } from '@/helpers/skins'
import { get_account_skins } from '@/helpers/skins'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const emit = defineEmits<{
	change: []
}>()

type MinecraftCredential = {
	profile: {
		id: string
		name: string
	}
	refresh_token?: string
}

const accounts: Ref<MinecraftCredential[]> = ref([])
const loginDisabled = ref(false)
const defaultUser = ref<string | undefined>()
const accountSkins = ref<Record<string, Skin>>({})
const accountHeadUrls = ref<Record<string, string>>({})
const renderedTextureKeys = ref<Record<string, string>>({})
let headRequest = 0

function revokeHeadUrls(urls: Record<string, string>) {
	for (const url of Object.values(urls)) URL.revokeObjectURL(url)
}

async function updateHeadUrls(skins: Record<string, Skin>) {
	const request = ++headRequest
	const validProfileIds = new Set(accounts.value.map((a) => a.profile.id))
	const retainedUrls: Record<string, string> = {}
	const retainedKeys: Record<string, string> = {}
	for (const [profileId, url] of Object.entries(accountHeadUrls.value)) {
		if (validProfileIds.has(profileId)) {
			retainedUrls[profileId] = url
			if (renderedTextureKeys.value[profileId]) {
				retainedKeys[profileId] = renderedTextureKeys.value[profileId]
			}
		} else {
			URL.revokeObjectURL(url)
		}
	}
	accountHeadUrls.value = retainedUrls
	renderedTextureKeys.value = retainedKeys

	await Promise.allSettled(
		Object.entries(skins).map(async ([profileId, skin]) => {
			try {
				if (!skin) return
				const currentUrl = accountHeadUrls.value[profileId]
				const currentKey = renderedTextureKeys.value[profileId]
				if (currentUrl && currentKey === skin.texture_key) {
					return
				}

				const url = await getPlayerHeadUrl(skin)
				if (request !== headRequest) {
					URL.revokeObjectURL(url)
					return
				}

				const previousUrl = accountHeadUrls.value[profileId]
				if (previousUrl && previousUrl !== url) URL.revokeObjectURL(previousUrl)
				renderedTextureKeys.value[profileId] = skin.texture_key
				accountHeadUrls.value = {
					...accountHeadUrls.value,
					[profileId]: url,
				}
			} catch (error) {
				console.warn(`Failed to render head for Minecraft account ${profileId}:`, error)
			}
		}),
	)
}

onUnmounted(() => {
	headRequest++
	revokeHeadUrls(accountHeadUrls.value)
})

async function refreshValues() {
	defaultUser.value = await get_default_user().catch(handleError)
	const userList = await users().catch(handleError)
	accounts.value = Array.isArray(userList) ? [...userList] : []
	accounts.value.sort((a, b) => (a.profile?.name ?? '').localeCompare(b.profile?.name ?? ''))

	let backendSkins: Record<string, Skin> = {}
	try {
		backendSkins = Object.fromEntries(
			(await get_account_skins()).map(({ profile_id, skin }) => [profile_id, skin]),
		)
	} catch (e) {
		console.warn('Failed to get account skins from backend:', e)
	}

	const mergedSkins: Record<string, Skin> = {
		...accountSkins.value,
		...backendSkins,
	}

	for (const account of accounts.value) {
		const profileId = account.profile?.id
		if (profileId && !mergedSkins[profileId]) {
			const activeSkin =
				(account as any).profile?.skins?.find((s: any) => s.state === 'ACTIVE') ??
				(account as any).profile?.skins?.[0]
			if (activeSkin?.url) {
				mergedSkins[profileId] = {
					texture_key:
						activeSkin.texture_key ||
						activeSkin.textureKey ||
						activeSkin.id ||
						`${profileId}-skin`,
					variant: activeSkin.variant || 'CLASSIC',
					texture: activeSkin.url,
					source: 'custom_external',
					is_equipped: true,
				}
			}
		}
	}

	accountSkins.value = mergedSkins
	await updateHeadUrls(mergedSkins)
}

async function setEquippedSkin(skin: Skin) {
	if (!defaultUser.value) return
	const profileId = defaultUser.value

	accountSkins.value = {
		...accountSkins.value,
		[profileId]: skin,
	}

	try {
		const request = ++headRequest
		const url = await getPlayerHeadUrl(skin, true)
		if (request !== headRequest) {
			URL.revokeObjectURL(url)
			return
		}

		const previousUrl = accountHeadUrls.value[profileId]
		if (previousUrl && previousUrl !== url) URL.revokeObjectURL(previousUrl)
		renderedTextureKeys.value[profileId] = skin.texture_key
		accountHeadUrls.value = {
			...accountHeadUrls.value,
			[profileId]: url,
		}
	} catch (error) {
		console.warn('Failed to get head render for equipped skin:', error)
	}
}

function setLoginDisabled(value: boolean) {
	loginDisabled.value = value
}

const offlineModal = ref<InstanceType<typeof OfflineLoginModal>>()

function openOfflineLogin() {
	offlineModal.value?.show()
}

async function handleOfflineLoggedIn(account: MinecraftCredential) {
	await refreshValues()
	if (account?.profile?.id) {
		await setAccount(account)
	}
}

defineExpose({
	refreshValues,
	setEquippedSkin,
	setLoginDisabled,
	login,
	openOfflineLogin,
	loginDisabled,
})

await refreshValues()

const selectedAccount = computed(() =>
	accounts.value.find((account) => account.profile.id === defaultUser.value),
)

const avatarUrl = computed(() => {
	return selectedAccount.value
		? getAccountAvatarUrl(selectedAccount.value)
		: 'https://launcher-files.modrinth.com/assets/steve_head.png'
})

function getAccountAvatarUrl(account: MinecraftCredential) {
	const profileId = account.profile?.id
	if (!profileId) return 'https://launcher-files.modrinth.com/assets/steve_head.png'

	const cachedUrl = accountHeadUrls.value[profileId]
	if (cachedUrl) return cachedUrl

	const skin = accountSkins.value[profileId]
	const isOffline = account.refresh_token === 'offline'

	if (!isOffline && skin?.texture_key && !skin.texture_key.startsWith('local-') && !skin.texture_key.startsWith('catalog-')) {
		return `https://mc-heads.net/avatar/${skin.texture_key}/128`
	}

	if (!isOffline) {
		return `https://mc-heads.net/avatar/${profileId}/128`
	}

	return 'https://launcher-files.modrinth.com/assets/steve_head.png'
}

async function setAccount(account: MinecraftCredential) {
	defaultUser.value = account.profile.id
	await set_default_user(account.profile.id).catch(handleError)
	await refreshValues()
	emit('change')
}

async function login() {
	loginDisabled.value = true
	const loggedIn = await login_flow().catch(handleSevereError)

	if (loggedIn) {
		await setAccount(loggedIn)
	}

	trackEvent('AccountLogIn')
	loginDisabled.value = false
}

async function logout(id: string) {
	await remove_user(id).catch(handleError)
	await refreshValues()
	if (!selectedAccount.value && accounts.value.length > 0) {
		await setAccount(accounts.value[0])
	} else {
		emit('change')
	}
	trackEvent('AccountLogOut')
}

useAppEvent('process', async (e) => {
	if (e.event === 'launched') {
		await refreshValues()
	}
})

const messages = defineMessages({
	notSignedIn: {
		id: 'minecraft-account.not-signed-in',
		defaultMessage: 'Not signed in',
	},
	addAccount: {
		id: 'minecraft-account.add-account',
		defaultMessage: 'Add account',
	},
	removeAccount: {
		id: 'minecraft-account.remove-account',
		defaultMessage: 'Remove account',
	},
	selectAccount: {
		id: 'minecraft-account.select-account',
		defaultMessage: 'Select account',
	},
	minecraftAccount: {
		id: 'minecraft-account.label',
		defaultMessage: 'Minecraft account',
	},
	signInToMinecraft: {
		id: 'minecraft-account.sign-in',
		defaultMessage: 'Sign in to Minecraft',
	},
	addOfflineAccount: {
		id: 'minecraft-account.add-offline-account',
		defaultMessage: 'Add offline account',
	},
	offlineBadge: {
		id: 'minecraft-account.offline-badge',
		defaultMessage: 'Offline',
	},
	offlineAccountLabel: {
		id: 'minecraft-account.offline-account-label',
		defaultMessage: 'Offline account',
	},
})
</script>
