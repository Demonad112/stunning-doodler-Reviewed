import { FileText, FolderSync, HardDrive, House, Settings } from '@lucide/vue'
import type { Component } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import HomePage from './pages/HomePage.vue'
import PlaceholderPage from './pages/PlaceholderPage.vue'
import SettingsPage from './pages/SettingsPage.vue'

export interface Section {
  path: string
  title: string
  summary: string
  icon: Component
}

/** The app's sections, in nav order. Settings sits at the bottom of the rail. */
export const sections: Section[] = [
  { path: '/', title: 'Home', summary: 'Start here.', icon: House },
  {
    path: '/compare',
    title: 'Compare & Record',
    summary:
      'Compare two folders by size, then record a copy between them and see every file that did not make it.',
    icon: FolderSync,
  },
  {
    path: '/cleanup',
    title: 'Disk Cleanup',
    summary: 'See what fills a drive and free space safely.',
    icon: HardDrive,
  },
  {
    path: '/reports',
    title: 'Reports',
    summary: 'Every compare, copy and cleanup report, ready to export for the client.',
    icon: FileText,
  },
]

export const settingsSection: Section = {
  path: '/settings',
  title: 'Settings',
  summary: 'Appearance and defaults.',
  icon: Settings,
}

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: HomePage },
    ...sections.slice(1).map((section) => ({
      path: section.path,
      component: PlaceholderPage,
      props: { section },
    })),
    { path: '/settings', component: SettingsPage },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
})
