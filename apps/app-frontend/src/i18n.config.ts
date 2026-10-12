import {
	buildLocaleMessages,
	createMessageCompiler,
	type CrowdinMessages,
	FALLBACK_LOCALE,
	LOCALES,
} from '@modrinth/ui'
import englishUi from '@modrinth/ui/src/locales/en-US/index.json'
import { createI18n } from 'vue-i18n'

import englishApp from './locales/en-US/index.json'

const localeUrls = import.meta.glob<string>('./locales/*/index.json', {
	eager: true,
	query: '?url',
	import: 'default',
})
const uiLocaleUrls = import.meta.glob<string>('../../../packages/ui/src/locales/*/index.json', {
	eager: true,
	query: '?url',
	import: 'default',
})

function applyLeetMCBranding(
	messages: Record<string, Record<string, string>>,
): Record<string, Record<string, string>> {
	return Object.fromEntries(
		Object.entries(messages).map(([locale, localeMessages]) => [
			locale,
			Object.fromEntries(
				Object.entries(localeMessages).map(([key, message]) => [
					key,
					message.replaceAll('Modrinth', 'LeetMC'),
				]),
			),
		]),
	)
}

const i18n = createI18n({
	legacy: false,
	locale: FALLBACK_LOCALE,
	fallbackLocale: FALLBACK_LOCALE,
	messageCompiler: createMessageCompiler(),
	missingWarn: false,
	fallbackWarn: false,
	messages: applyLeetMCBranding(
		buildLocaleMessages({
			'./app/en-US/index.json': { default: englishApp },
			'./ui/en-US/index.json': { default: englishUi },
		}),
	),
})

const pendingLocales = new Map<string, Promise<Record<string, string>>>()
let localeRequest = 0

async function fetchMessages(url: string): Promise<CrowdinMessages> {
	const response = await fetch(url)
	if (!response.ok) throw new Error(`Could not load translations: ${response.status}`)
	return response.json()
}

export async function setLocale(requestedLocale: string): Promise<void> {
	const request = ++localeRequest
	const locale =
		requestedLocale === FALLBACK_LOCALE ||
		LOCALES.some((candidate) => candidate.code === requestedLocale)
			? requestedLocale
			: FALLBACK_LOCALE
	if (locale !== FALLBACK_LOCALE && locale !== i18n.global.locale.value) {
		let pending = pendingLocales.get(locale)
		if (!pending) {
			pending = Promise.all([
				fetchMessages(localeUrls[`./locales/${locale}/index.json`]),
				fetchMessages(uiLocaleUrls[`../../../packages/ui/src/locales/${locale}/index.json`]),
			])
				.then(
					([app, ui]) =>
						applyLeetMCBranding(
							buildLocaleMessages({
								[`./app/${locale}/index.json`]: { default: app },
								[`./ui/${locale}/index.json`]: { default: ui },
							}),
						)[locale],
				)
				.finally(() => pendingLocales.delete(locale))
			pendingLocales.set(locale, pending)
		}
		const messages = await pending
		if (request !== localeRequest) return
		i18n.global.setLocaleMessage(locale, messages)
	}
	if (request !== localeRequest) return
	const previous = i18n.global.locale.value
	i18n.global.locale.value = locale
	if (previous !== locale && previous !== FALLBACK_LOCALE) {
		i18n.global.setLocaleMessage(previous, {})
	}
}

export default i18n
