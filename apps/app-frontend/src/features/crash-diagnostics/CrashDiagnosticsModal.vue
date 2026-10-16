<script setup lang="ts">
import {
	BugIcon,
	CheckIcon,
	CoffeeIcon,
	CopyIcon,
	FileTextIcon,
	MemoryStickIcon,
	PlayIcon,
	RotateCounterClockwiseIcon,
	SpinnerIcon,
	TriangleAlertIcon,
	WrenchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { login as loginFlow, set_default_user } from '@/helpers/auth'
import {
	edit,
	get as getInstance,
	get_content_items,
	run,
	sync_content_files,
	toggle_disable_project,
} from '@/helpers/instance'
import {
	install_duplicate_instance,
	install_existing_instance,
	installJobInstanceId,
	wait_for_install_job,
} from '@/helpers/install'
import { get_max_memory } from '@/helpers/jre'
import { type AppSettings, get as getSettings } from '@/helpers/settings'
import type { GameInstance } from '@/helpers/types'
import { injectAppEvents } from '@/providers/app-events'

import {
	analyseCrash,
	type CrashAnalysis,
	resolveSuspectContent,
	type SuspectContent,
} from './crash-diagnostics'

type FixAction = 'disable_mod' | 'increase_memory' | 'reset_java' | 'repair' | 'reauthenticate'

interface ShowPayload {
	instanceId: string
	instanceName?: string
	log: string
	processMessage?: string
}

const messages = defineMessages({
	header: {
		id: 'app.crash-diagnostics.header',
		defaultMessage: 'What happened',
	},
	intro: {
		id: 'app.crash-diagnostics.intro',
		defaultMessage: 'LeetMC analysed the game log and found the most likely cause.',
	},
	categoryMemory: {
		id: 'app.crash-diagnostics.category.memory',
		defaultMessage: 'Minecraft ran out of memory',
	},
	categoryJava: {
		id: 'app.crash-diagnostics.category.java',
		defaultMessage: 'The selected Java version is incompatible',
	},
	categoryModConflict: {
		id: 'app.crash-diagnostics.category.mod-conflict',
		defaultMessage: 'A mod failed or conflicts with another mod',
	},
	categoryLoader: {
		id: 'app.crash-diagnostics.category.loader',
		defaultMessage: 'The mod loader or game version is incompatible',
	},
	categoryAuthentication: {
		id: 'app.crash-diagnostics.category.authentication',
		defaultMessage: 'Minecraft authentication failed',
	},
	categoryCorruptFiles: {
		id: 'app.crash-diagnostics.category.corrupt-files',
		defaultMessage: 'Some game files are damaged or incomplete',
	},
	categoryGraphics: {
		id: 'app.crash-diagnostics.category.graphics',
		defaultMessage: 'Minecraft could not initialise graphics',
	},
	categoryUnknown: {
		id: 'app.crash-diagnostics.category.unknown',
		defaultMessage: 'Minecraft closed unexpectedly',
	},
	descriptionMemory: {
		id: 'app.crash-diagnostics.description.memory',
		defaultMessage: 'The log contains a memory allocation error. Increasing the limit should help.',
	},
	descriptionJava: {
		id: 'app.crash-diagnostics.description.java',
		defaultMessage: 'This instance is using a Java runtime that cannot load the selected game or mods.',
	},
	descriptionModConflict: {
		id: 'app.crash-diagnostics.description.mod-conflict',
		defaultMessage: 'A mod crashed while loading. Disable the detected mod and try launching again.',
	},
	descriptionLoader: {
		id: 'app.crash-diagnostics.description.loader',
		defaultMessage: 'At least one installed mod does not match the current loader or Minecraft version.',
	},
	descriptionAuthentication: {
		id: 'app.crash-diagnostics.description.authentication',
		defaultMessage: 'Your session is no longer valid or the authentication service rejected it.',
	},
	descriptionCorruptFiles: {
		id: 'app.crash-diagnostics.description.corrupt-files',
		defaultMessage: 'LeetMC can verify the installation and download the missing files again.',
	},
	descriptionGraphics: {
		id: 'app.crash-diagnostics.description.graphics',
		defaultMessage: 'The graphics driver or OpenGL support prevented the game window from opening.',
	},
	descriptionUnknown: {
		id: 'app.crash-diagnostics.description.unknown',
		defaultMessage: 'The log does not contain a reliable signature. Open the logs for the full details.',
	},
	suspectMod: {
		id: 'app.crash-diagnostics.suspect-mod',
		defaultMessage: 'Likely culprit: {mod}',
	},
	confidenceHigh: {
		id: 'app.crash-diagnostics.confidence.high',
		defaultMessage: 'High confidence',
	},
	confidenceMedium: {
		id: 'app.crash-diagnostics.confidence.medium',
		defaultMessage: 'Probable cause',
	},
	confidenceLow: {
		id: 'app.crash-diagnostics.confidence.low',
		defaultMessage: 'Needs review',
	},
	restorePointNotice: {
		id: 'app.crash-diagnostics.restore-point-notice',
		defaultMessage: 'Before changing the instance, LeetMC will automatically create a restore point.',
	},
	disableMod: {
		id: 'app.crash-diagnostics.action.disable-mod',
		defaultMessage: 'Back up and disable mod',
	},
	increaseMemory: {
		id: 'app.crash-diagnostics.action.increase-memory',
		defaultMessage: 'Back up and increase memory',
	},
	resetJava: {
		id: 'app.crash-diagnostics.action.reset-java',
		defaultMessage: 'Back up and select Java automatically',
	},
	repairFiles: {
		id: 'app.crash-diagnostics.action.repair-files',
		defaultMessage: 'Back up and repair files',
	},
	signInAgain: {
		id: 'app.crash-diagnostics.action.sign-in-again',
		defaultMessage: 'Sign in again',
	},
	viewLogs: {
		id: 'app.crash-diagnostics.action.view-logs',
		defaultMessage: 'View logs',
	},
	openRestorePoint: {
		id: 'app.crash-diagnostics.action.open-restore-point',
		defaultMessage: 'Open restore point',
	},
	tryAgain: {
		id: 'app.crash-diagnostics.action.try-again',
		defaultMessage: 'Play again',
	},
	evidence: {
		id: 'app.crash-diagnostics.evidence',
		defaultMessage: 'Technical details',
	},
	exitCode: {
		id: 'app.crash-diagnostics.exit-code',
		defaultMessage: 'Exit code: {code}',
	},
	creatingRestorePoint: {
		id: 'app.crash-diagnostics.status.creating-restore-point',
		defaultMessage: 'Creating a restore point…',
	},
	applyingFix: {
		id: 'app.crash-diagnostics.status.applying-fix',
		defaultMessage: 'Applying the fix…',
	},
	fixed: {
		id: 'app.crash-diagnostics.status.fixed',
		defaultMessage: 'Fix applied. You can try launching the instance again.',
	},
	restorePointName: {
		id: 'app.crash-diagnostics.restore-point-name',
		defaultMessage: '{instance} — Before automatic repair',
	},
	copyDetails: {
		id: 'app.crash-diagnostics.action.copy-details',
		defaultMessage: 'Copy details',
	},
})

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const appEvents = injectAppEvents()
const router = useRouter()
const modal = ref<InstanceType<typeof NewModal>>()
const instance = ref<GameInstance | null>(null)
const instanceId = ref('')
const instanceName = ref('')
const analysis = ref<CrashAnalysis | null>(null)
const suspect = ref<SuspectContent | null>(null)
const restorePointId = ref<string | null>(null)
const busy = ref(false)
const status = ref<'idle' | 'backup' | 'fixing' | 'fixed'>('idle')

const categoryMessages = {
	memory: messages.categoryMemory,
	java: messages.categoryJava,
	mod_conflict: messages.categoryModConflict,
	loader: messages.categoryLoader,
	authentication: messages.categoryAuthentication,
	corrupt_files: messages.categoryCorruptFiles,
	graphics: messages.categoryGraphics,
	unknown: messages.categoryUnknown,
} as const

const descriptionMessages = {
	memory: messages.descriptionMemory,
	java: messages.descriptionJava,
	mod_conflict: messages.descriptionModConflict,
	loader: messages.descriptionLoader,
	authentication: messages.descriptionAuthentication,
	corrupt_files: messages.descriptionCorruptFiles,
	graphics: messages.descriptionGraphics,
	unknown: messages.descriptionUnknown,
} as const

const categoryTitle = computed(() =>
	formatMessage(categoryMessages[analysis.value?.category ?? 'unknown']),
)
const categoryDescription = computed(() =>
	formatMessage(descriptionMessages[analysis.value?.category ?? 'unknown']),
)
const confidenceLabel = computed(() => {
	switch (analysis.value?.confidence) {
		case 'high':
			return formatMessage(messages.confidenceHigh)
		case 'medium':
			return formatMessage(messages.confidenceMedium)
		default:
			return formatMessage(messages.confidenceLow)
	}
})
const primaryAction = computed<FixAction | null>(() => {
	switch (analysis.value?.category) {
		case 'memory':
			return 'increase_memory'
		case 'java':
			return 'reset_java'
		case 'mod_conflict':
			return suspect.value ? 'disable_mod' : null
		case 'loader':
			return suspect.value ? 'disable_mod' : null
		case 'corrupt_files':
			return 'repair'
		case 'authentication':
			return 'reauthenticate'
		default:
			return null
	}
})
const primaryActionLabel = computed(() => {
	switch (primaryAction.value) {
		case 'disable_mod':
			return formatMessage(messages.disableMod)
		case 'increase_memory':
			return formatMessage(messages.increaseMemory)
		case 'reset_java':
			return formatMessage(messages.resetJava)
		case 'repair':
			return formatMessage(messages.repairFiles)
		case 'reauthenticate':
			return formatMessage(messages.signInAgain)
		default:
			return ''
	}
})
const statusLabel = computed(() => {
	if (status.value === 'backup') return formatMessage(messages.creatingRestorePoint)
	if (status.value === 'fixing') return formatMessage(messages.applyingFix)
	if (status.value === 'fixed') return formatMessage(messages.fixed)
	return ''
})
const details = computed(() => {
	const evidence = analysis.value?.evidence.join('\n') ?? ''
	return [categoryTitle.value, suspect.value?.fileName, evidence].filter(Boolean).join('\n\n')
})

async function show(payload: ShowPayload) {
	instanceId.value = payload.instanceId
	instanceName.value = payload.instanceName ?? ''
	const diagnosis = analyseCrash(payload.log, payload.processMessage)
	analysis.value = diagnosis
	suspect.value = null
	restorePointId.value = null
	status.value = 'idle'
	busy.value = false
	modal.value?.show()

	try {
		instance.value = await getInstance(payload.instanceId)
		instanceName.value = instance.value?.name ?? instanceName.value
		if (diagnosis.category === 'mod_conflict' || diagnosis.category === 'loader') {
			const content = await get_content_items(
				payload.instanceId,
				'stale_while_revalidate_skip_offline',
			)
			suspect.value = resolveSuspectContent(content, payload.log, diagnosis.suspectTokens)
		}
	} catch {
		// Diagnosis remains useful even if metadata enrichment is unavailable.
	}
}

async function createRestorePoint() {
	const currentInstance = instance.value ?? (await getInstance(instanceId.value))
	if (!currentInstance) throw new Error('Instance is no longer available')
	instance.value = currentInstance
	status.value = 'backup'

	const job = await install_duplicate_instance(currentInstance.id)
	const completed = await wait_for_install_job(appEvents, job.job_id)
	const backupId = installJobInstanceId(completed)
	if (backupId) {
		restorePointId.value = backupId
		await edit(backupId, {
			name: formatMessage(messages.restorePointName, { instance: currentInstance.name }),
		})
	}
}

async function applyPrimaryFix() {
	const action = primaryAction.value
	if (!action || busy.value) return
	busy.value = true

	try {
		if (action === 'reauthenticate') {
			const loggedIn = await loginFlow()
			if (loggedIn) await set_default_user(loggedIn.profile.id)
			status.value = 'fixed'
			return
		}

		await createRestorePoint()
		status.value = 'fixing'

		if (action === 'disable_mod' && suspect.value) {
			await toggle_disable_project(instanceId.value, suspect.value.filePath, false)
		} else if (action === 'reset_java') {
			await edit(instanceId.value, { java_path: null } as unknown as Partial<GameInstance>)
		} else if (action === 'increase_memory') {
			await increaseMemory()
		} else if (action === 'repair') {
			await sync_content_files(instanceId.value)
			const repairJob = await install_existing_instance(instanceId.value, true)
			await wait_for_install_job(appEvents, repairJob.job_id)
		}

		status.value = 'fixed'
		addNotification({ type: 'success', title: formatMessage(messages.fixed) })
	} catch (error) {
		status.value = 'idle'
		handleError(error)
	} finally {
		busy.value = false
	}
}

async function increaseMemory() {
	const currentInstance = instance.value
	if (!currentInstance) throw new Error('Instance is no longer available')
	const settings = (await getSettings()) as unknown as AppSettings
	const currentMaximum = currentInstance.memory?.maximum ?? settings.memory.maximum
	const systemMaximum = Math.max(512, Math.floor(Number(await get_max_memory()) / 1024))
	const nextMaximum = Math.min(systemMaximum, Math.max(4096, currentMaximum + 1024))
	if (nextMaximum <= currentMaximum) throw new Error('No additional system memory is available')
	await edit(instanceId.value, { memory: { maximum: nextMaximum } })
}

async function viewLogs() {
	modal.value?.hide()
	await router.push(`/instance/${encodeURIComponent(instanceId.value)}/logs`)
}

async function openRestorePoint() {
	if (!restorePointId.value) return
	modal.value?.hide()
	await router.push(`/instance/${encodeURIComponent(restorePointId.value)}`)
}

async function tryAgain() {
	if (busy.value) return
	busy.value = true
	try {
		await run(instanceId.value)
		modal.value?.hide()
	} catch (error) {
		handleError(error)
	} finally {
		busy.value = false
	}
}

async function copyDetails() {
	await navigator.clipboard.writeText(details.value)
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="warning" max-width="620px">
		<div class="flex min-w-0 flex-col gap-4">
			<div class="flex items-start gap-3 rounded-2xl border border-solid border-orange bg-bg-orange p-4">
				<div class="grid size-10 shrink-0 place-items-center rounded-xl bg-highlight-orange text-orange">
					<MemoryStickIcon v-if="analysis?.category === 'memory'" class="size-5" />
					<CoffeeIcon v-else-if="analysis?.category === 'java'" class="size-5" />
					<BugIcon v-else-if="analysis?.category === 'mod_conflict'" class="size-5" />
					<TriangleAlertIcon v-else class="size-5" />
				</div>
				<div class="flex min-w-0 flex-1 flex-col gap-1">
					<span v-if="instanceName" class="truncate text-xs font-semibold text-secondary">
						{{ instanceName }}
					</span>
					<div class="flex flex-wrap items-center gap-2">
						<h2 class="m-0 text-lg font-bold text-contrast">{{ categoryTitle }}</h2>
						<span class="rounded-full bg-surface-2 px-2 py-0.5 text-xs font-semibold text-secondary">
							{{ confidenceLabel }}
						</span>
					</div>
					<p class="m-0 text-sm leading-5 text-secondary">{{ categoryDescription }}</p>
					<p v-if="suspect" class="m-0 pt-1 text-sm font-semibold text-orange">
						{{ formatMessage(messages.suspectMod, { mod: suspect.name }) }}
					</p>
				</div>
			</div>

			<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.intro) }}</p>

			<div
				v-if="primaryAction && primaryAction !== 'reauthenticate' && status !== 'fixed'"
				class="flex items-start gap-2 rounded-xl bg-surface-2 px-3 py-2.5 text-sm text-secondary"
			>
				<RotateCounterClockwiseIcon class="mt-0.5 size-4 shrink-0 text-orange" />
				<span>{{ formatMessage(messages.restorePointNotice) }}</span>
			</div>

			<div
				v-if="statusLabel"
				class="flex items-center gap-2 rounded-xl bg-surface-2 px-3 py-2.5 text-sm font-semibold"
				:class="status === 'fixed' ? 'text-green' : 'text-contrast'"
			>
				<CheckIcon v-if="status === 'fixed'" class="size-4" />
				<SpinnerIcon v-else class="size-4 animate-spin" />
				{{ statusLabel }}
			</div>

			<details v-if="analysis?.evidence.length || analysis?.exitCode !== null" class="group">
				<summary class="cursor-pointer select-none text-sm font-semibold text-secondary">
					{{ formatMessage(messages.evidence) }}
				</summary>
				<div class="mt-2 flex min-w-0 flex-col gap-2 rounded-xl bg-surface-2 p-3">
					<span v-if="analysis?.exitCode !== null" class="text-xs font-semibold text-secondary">
						{{ formatMessage(messages.exitCode, { code: analysis?.exitCode }) }}
					</span>
					<code
						v-for="line in analysis?.evidence"
						:key="line"
						class="block max-w-full whitespace-pre-wrap break-words text-xs leading-5 text-secondary"
						>{{ line }}</code
					>
				</div>
			</details>
		</div>

		<template #actions>
			<div class="flex w-full flex-wrap items-center justify-end gap-2">
				<Button v-tooltip="formatMessage(messages.copyDetails)" :disabled="!details" @click="copyDetails">
					<CopyIcon />
				</Button>
				<Button type="outlined" :disabled="busy" @click="viewLogs">
					<FileTextIcon />
					{{ formatMessage(messages.viewLogs) }}
				</Button>
				<Button v-if="status === 'fixed' && restorePointId" type="outlined" @click="openRestorePoint">
					<RotateCounterClockwiseIcon />
					{{ formatMessage(messages.openRestorePoint) }}
				</Button>
				<Button
					v-if="status === 'fixed'"
					type="colored"
					color="brand"
					:disabled="busy"
					@click="tryAgain"
				>
					<SpinnerIcon v-if="busy" class="animate-spin" />
					<PlayIcon v-else />
					{{ formatMessage(messages.tryAgain) }}
				</Button>
				<Button v-if="!primaryAction || status === 'fixed'" type="outlined" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.closeButton) }}
				</Button>
				<Button
					v-else
					type="colored"
					color="brand"
					:disabled="busy"
					@click="applyPrimaryFix"
				>
					<SpinnerIcon v-if="busy" class="animate-spin" />
					<PlayIcon v-else-if="primaryAction === 'reauthenticate'" />
					<WrenchIcon v-else />
					{{ primaryActionLabel }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
