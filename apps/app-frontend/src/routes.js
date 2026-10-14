import { createRouter, createWebHistory } from 'vue-router'

const loadHome = () => import('@/pages/Index.vue')
const loadRooms = () => import('@/pages/Rooms.vue')
const loadBrowse = () => import('@/pages/Browse.vue')
const loadSkins = () => import('@/pages/Skins.vue')
const loadScreenshots = () => import('@/pages/Screenshots.vue')
const loadUser = () => import('@/pages/User.vue')
const loadWelcome = () => import('@/pages/Welcome.vue')
const loadProject = () => import('@/pages/project/Index.vue')
const loadProjectDescription = () => import('@/pages/project/Description.vue')
const loadProjectVersions = () => import('@/pages/project/Versions.vue')
const loadProjectVersion = () => import('@/pages/project/Version.vue')
const loadProjectGallery = () => import('@/pages/project/Gallery.vue')
const loadInstance = () => import('@/pages/instance/layout.vue')
const loadInstanceWorlds = () => import('@/pages/instance/worlds/index.vue')
const loadInstanceContent = () => import('@/pages/instance/content/index.vue')
const loadInstanceFiles = () => import('@/pages/instance/files/index.vue')
const loadInstanceScreenshots = () => import('@/pages/instance/screenshots/index.vue')
const loadInstanceLogs = () => import('@/pages/instance/logs/index.vue')

async function preloadRoutes(loaders) {
	await Promise.allSettled(loaders.map((load) => load()))
}

export function preloadPrimaryRoutes() {
	return preloadRoutes([loadRooms, loadBrowse, loadInstance, loadInstanceContent])
}

export function preloadSecondaryRoutes() {
	return preloadRoutes([
		loadScreenshots,
		loadProject,
		loadProjectDescription,
		loadProjectVersions,
		loadInstanceWorlds,
		loadInstanceFiles,
		loadInstanceScreenshots,
		loadInstanceLogs,
	])
}

export function preloadRemainingRoutes() {
	return preloadRoutes([
		loadSkins,
		loadUser,
		loadProjectVersion,
		loadProjectGallery,
		loadWelcome,
	])
}

const router = new createRouter({
	history: createWebHistory(),
	routes: [
		{
			path: '/',
			name: 'Home',
			component: loadHome,
		},
		{
			path: '/welcome',
			name: 'Welcome',
			component: loadWelcome,
		},
		{
			path: '/rooms',
			name: 'Rooms',
			component: loadRooms,
		},
		{
			path: '/browse/:projectType',
			name: 'Discover content',
			component: loadBrowse,
		},
		{
			path: '/skins',
			name: 'Skin selector',
			component: loadSkins,
		},
		{
			path: '/screenshots',
			name: 'Screenshots',
			component: loadScreenshots,
		},
		{
			path: '/user/:user/:projectType?',
			name: 'User',
			component: loadUser,
		},
		{
			path: '/:projectType(mod|plugin|datapack|resourcepack|shader|modpack)/:id/:rest(.*)*',
			redirect: (to) => {
				const rest = to.params.rest ? `/${[].concat(to.params.rest).join('/')}` : ''
				return `/project/${to.params.id}${rest}${to.hash}`
			},
		},
		{
			path: '/project/:id',
			name: 'Project',
			component: loadProject,
			props: true,
			children: [
				{
					path: '',
					name: 'Description',
					component: loadProjectDescription,
				},
				{
					path: 'versions',
					name: 'Versions',
					component: loadProjectVersions,
				},
				{
					path: 'version/:version',
					name: 'Version',
					component: loadProjectVersion,
					props: true,
				},
				{
					path: 'gallery',
					name: 'Gallery',
					component: loadProjectGallery,
				},
			],
		},
		{
			path: '/instance/:id',
			name: 'Instance',
			component: loadInstance,
			children: [
				{
					path: 'worlds',
					name: 'InstanceWorlds',
					component: loadInstanceWorlds,
				},
				{
					path: '',
					name: 'InstanceContent',
					component: loadInstanceContent,
				},
				{
					path: 'projects/:type',
					name: 'InstanceContentFilter',
					component: loadInstanceContent,
				},
				{
					path: 'files',
					name: 'InstanceFiles',
					component: loadInstanceFiles,
				},
				{
					path: 'screenshots',
					name: 'InstanceScreenshots',
					component: loadInstanceScreenshots,
				},
				{
					path: 'logs',
					name: 'InstanceLogs',
					component: loadInstanceLogs,
					meta: {
						renderMode: 'fixed',
					},
				},
			],
		},
	],
	linkActiveClass: 'router-link-active',
	linkExactActiveClass: 'router-link-exact-active',
	scrollBehavior(to, from) {
		if (to.path === from.path || to.name === 'Home') return
		// Sometimes Vue's scroll behavior is not working as expected, so we need to manually scroll to top (especially on Linux)
		document.querySelector('.app-viewport')?.scrollTo(0, 0)
		return {
			el: '.app-viewport',
			top: 0,
		}
	},
})

export default router
