import type { ContentItem } from '@modrinth/ui'

export type CrashCategory =
	| 'memory'
	| 'java'
	| 'mod_conflict'
	| 'loader'
	| 'authentication'
	| 'corrupt_files'
	| 'graphics'
	| 'unknown'

export type CrashConfidence = 'high' | 'medium' | 'low'

export interface CrashAnalysis {
	category: CrashCategory
	confidence: CrashConfidence
	evidence: string[]
	suspectTokens: string[]
	exitCode: number | null
}

export interface SuspectContent {
	name: string
	filePath: string
	fileName: string
}

const CATEGORY_PATTERNS: Array<{
	category: Exclude<CrashCategory, 'unknown'>
	patterns: RegExp[]
}> = [
	{
		category: 'memory',
		patterns: [
			/java\.lang\.OutOfMemoryError/i,
			/GC overhead limit exceeded/i,
			/could not reserve enough space for object heap/i,
			/unable to create (?:new )?native thread/i,
			/native memory allocation .* failed/i,
		],
	},
	{
		category: 'java',
		patterns: [
			/UnsupportedClassVersionError/i,
			/has been compiled by a more recent version of the Java Runtime/i,
			/only recognizes class file versions up to/i,
			/Unable to locate a Java Runtime/i,
			/Could not find or load main class/i,
			/JNI error has occurred/i,
		],
	},
	{
		category: 'loader',
		patterns: [
			/requires (?:fabricloader|forge|neoforge|quilt_loader)/i,
			/Mod resolution encountered an incompatible mod set/i,
			/Loading errors encountered.*loader/i,
			/Unsupported loader/i,
			/wrong (?:Minecraft|game|loader) version/i,
		],
	},
	{
		category: 'mod_conflict',
		patterns: [
			/Incompatible mods found/i,
			/ModResolutionException/i,
			/MixinApplyError/i,
			/MixinTransformerError/i,
			/Failed to load mod/i,
			/Failure message:.*(?:mod|jar)/i,
			/Missing or unsupported mandatory dependencies/i,
			/duplicate mods/i,
		],
	},
	{
		category: 'authentication',
		patterns: [
			/Invalid session/i,
			/Failed to verify username/i,
			/Authentication servers are down/i,
			/Not authenticated with Minecraft/i,
			/HTTP (?:Response Code: )?(?:401|403)/i,
			/unauthorized/i,
		],
	},
	{
		category: 'corrupt_files',
		patterns: [
			/zip END header not found/i,
			/invalid LOC header/i,
			/error in opening zip file/i,
			/checksum (?:did not match|mismatch)/i,
			/(?:corrupt|damaged) (?:jar|file|archive)/i,
			/failed to download file/i,
		],
	},
	{
		category: 'graphics',
		patterns: [
			/GLFW error/i,
			/OpenGL .* not supported/i,
			/Failed to create (?:window|display)/i,
			/Pixel format not accelerated/i,
			/EXCEPTION_ACCESS_VIOLATION.*(?:nvoglv|atio|ig)/i,
		],
	},
]

const IMPORTANT_LINE =
	/(?:fatal|error|exception|caused by|crash|failed|failure|incompatible|outofmemory|unsupported|corrupt|invalid session)/i
const IGNORED_SUSPECTS = new Set([
	'java',
	'minecraft',
	'fabric',
	'fabricloader',
	'forge',
	'neoforge',
	'quilt',
	'mixin',
])

export function parseProcessExit(message: string): { success: boolean | null; code: number | null } {
	const successMatch = message.match(/success=(true|false)/i)
	const codeMatch = message.match(/code=(-?\d+|none)/i)
	return {
		success: successMatch ? successMatch[1].toLowerCase() === 'true' : null,
		code: codeMatch && codeMatch[1] !== 'none' ? Number(codeMatch[1]) : null,
	}
}

export function shouldRunCrashDiagnostics(log: string, processMessage: string): boolean {
	if (/manually_stopped=true/i.test(processMessage)) return false
	const exit = parseProcessExit(processMessage)
	if (exit.success === true) return false
	if (exit.success === false) return true

	return CATEGORY_PATTERNS.some(({ patterns }) => patterns.some((pattern) => pattern.test(log)))
}

export function analyseCrash(log: string, processMessage = ''): CrashAnalysis {
	const lines = log.split(/\r?\n/)
	const importantLines = lines.filter((line) => IMPORTANT_LINE.test(line)).slice(-160)
	const searchable = importantLines.length > 0 ? importantLines.join('\n') : lines.slice(-500).join('\n')
	let bestCategory: CrashCategory = 'unknown'
	let bestScore = 0

	for (const { category, patterns } of CATEGORY_PATTERNS) {
		const score = patterns.reduce((total, pattern) => total + (pattern.test(searchable) ? 1 : 0), 0)
		if (score > bestScore) {
			bestCategory = category
			bestScore = score
		}
	}

	const matchingPatterns = CATEGORY_PATTERNS.find(({ category }) => category === bestCategory)?.patterns ?? []
	const evidence = importantLines
		.filter((line) => matchingPatterns.some((pattern) => pattern.test(line)))
		.concat(importantLines.slice(-4))
		.map(cleanEvidenceLine)
		.filter((line, index, all) => line.length > 0 && all.indexOf(line) === index)
		.slice(0, 4)

	return {
		category: bestCategory,
		confidence: bestScore >= 2 ? 'high' : bestScore === 1 ? 'medium' : 'low',
		evidence,
		suspectTokens: extractSuspectTokens(searchable),
		exitCode: parseProcessExit(processMessage).code,
	}
}

export function resolveSuspectContent(
	items: ContentItem[],
	log: string,
	tokens: string[],
): SuspectContent | null {
	const criticalLines = log
		.split(/\r?\n/)
		.filter((line) => IMPORTANT_LINE.test(line))
		.slice(-240)
		.join('\n')
		.toLowerCase()
	const normalizedTokens = tokens.map(normalizeToken).filter((token) => token.length >= 3)
	let best: { item: ContentItem; score: number } | null = null

	for (const item of items) {
		if (!item.file_path || !item.enabled || item.project_type !== 'mod') continue

		const fileName = item.file_name.toLowerCase()
		const fileStem = normalizeToken(fileName.replace(/\.jar$/i, ''))
		const slug = normalizeToken(item.project?.slug ?? '')
		const title = normalizeToken(item.project?.title ?? item.embedded_metadata?.name ?? '')
		let score = 0

		if (criticalLines.includes(fileName)) score += 12
		if (normalizedTokens.some((token) => token === fileStem || fileStem.startsWith(`${token}-`))) {
			score += 10
		}
		if (slug && normalizedTokens.includes(slug)) score += 9
		if (title && normalizedTokens.includes(title)) score += 8
		if (slug.length >= 4 && criticalLines.includes(slug)) score += 4
		if (title.length >= 4 && criticalLines.includes(title)) score += 3

		if (!best || score > best.score) best = { item, score }
	}

	if (!best || best.score < 8 || !best.item.file_path) return null
	return {
		name: best.item.project?.title ?? best.item.embedded_metadata?.name ?? best.item.file_name,
		filePath: best.item.file_path,
		fileName: best.item.file_name,
	}
}

function extractSuspectTokens(log: string): string[] {
	const tokens = new Set<string>()
	const patterns = [
		/(?:suspected mod|mod id|from mod)\s*[:=]?\s*["']?([\w.-]{3,})/gi,
		/([\w.+-]{3,})\.mixins(?:\.json)?/gi,
		/([\w.+@() -]{3,}\.jar)/gi,
	]

	for (const pattern of patterns) {
		for (const match of log.matchAll(pattern)) {
			const token = normalizeToken(match[1].replace(/\.jar$/i, ''))
			if (token.length >= 3 && !IGNORED_SUSPECTS.has(token)) tokens.add(token)
		}
	}

	return [...tokens].slice(0, 20)
}

function normalizeToken(value: string): string {
	return value
		.toLowerCase()
		.trim()
		.replace(/[^a-z0-9а-яё._+-]+/gi, '-')
		.replace(/^-+|-+$/g, '')
}

function cleanEvidenceLine(line: string): string {
	return line
		.replace(/\x1b\[[0-9;]*m/g, '')
		.replace(/^\s*(?:\[[^\]]+\]\s*)+/, '')
		.trim()
		.slice(0, 280)
}
