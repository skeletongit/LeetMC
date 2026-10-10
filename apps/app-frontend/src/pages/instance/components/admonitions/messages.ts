import { defineMessages } from '@modrinth/ui'

export const instanceAdmonitionsMessages = defineMessages({
	sharedInstanceWrongAccountHeader: {
		id: 'app.instance.shared-instance-wrong-account.warning-header',
		defaultMessage: 'You are using the wrong Modrinth account',
	},
	sharedInstanceSignedOutHeader: {
		id: 'app.instance.shared-instance-wrong-account.signed-out-header',
		defaultMessage: 'You need to sign in to Modrinth',
	},
	sharedInstanceWrongAccountSignInAs: {
		id: 'app.instance.shared-instance-wrong-account.sign-in-as-label',
		defaultMessage: 'Sign in as',
	},
	sharedInstanceWrongAccountUserBody: {
		id: 'app.instance.shared-instance-wrong-account.user-admonition-body-v2',
		defaultMessage: 'to receive updates for this shared instance.',
	},
	sharedInstanceWrongAccountOwnerBody: {
		id: 'app.instance.shared-instance-wrong-account.owner-admonition-body-v2',
		defaultMessage: "to manage this shared instance. You won't be able to push updates to users.",
	},
	sharedInstanceWrongAccountFallbackUsername: {
		id: 'app.instance.shared-instance-wrong-account.fallback-username',
		defaultMessage: 'the linked account',
	},
})
