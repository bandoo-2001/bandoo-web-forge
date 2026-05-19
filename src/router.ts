import { createRouter, createWebHashHistory } from 'vue-router'
import DashboardPage from './pages/DashboardPage.vue'
import AutomationPage from './pages/AutomationPage.vue'
import SettingsPage from './pages/SettingsPage.vue'

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', name: 'dashboard', component: DashboardPage },
    { path: '/automation', name: 'automation', component: AutomationPage },
    { path: '/settings', name: 'settings', component: SettingsPage },
  ],
})
