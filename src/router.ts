import { createRouter, createWebHashHistory } from 'vue-router'
import DashboardPage from './pages/DashboardPage.vue'
import AutomationPage from './pages/AutomationPage.vue'
import ScriptsPage from './pages/ScriptsPage.vue'
import AiPage from './pages/AiPage.vue'
import SettingsPage from './pages/SettingsPage.vue'
import WebAppShellPage from './pages/WebAppShellPage.vue'

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'dashboard', component: DashboardPage },
    { path: '/automation', name: 'automation', component: AutomationPage },
    { path: '/scripts', name: 'scripts', component: ScriptsPage },
    { path: '/ai', name: 'ai', component: AiPage },
    { path: '/settings', name: 'settings', component: SettingsPage },
    { path: '/shell/:id', name: 'webapp-shell', component: WebAppShellPage, meta: { shell: true } },
  ],
})
