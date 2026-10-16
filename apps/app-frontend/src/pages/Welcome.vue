<script setup lang="ts">
import { ModrinthIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'

import WelcomeScreen from '@/components/ui/WelcomeScreen.vue'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'

defineOptions({ name: 'WelcomePage' })

const { formatMessage } = useVIntl()
const messages = defineMessages({
	welcome: {
		id: 'app.welcome-screen.title',
		defaultMessage: 'Welcome to LeetMC',
	},
})

useRootBreadcrumb({
	slot: 'root',
	id: 'welcome',
	label: formatMessage(messages.welcome),
	to: '/welcome',
	visual: { type: 'icon', component: ModrinthIcon },
})
</script>

<template>
	<div class="welcome-page box-border h-full">
		<WelcomeScreen />
	</div>
</template>
