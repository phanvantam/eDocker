import { createRouter, createWebHistory } from 'vue-router';
import DashboardView from '../views/DashboardView.vue';
import ContainersView from '../views/ContainersView.vue';
import ImagesView from '../views/ImagesView.vue';
import TemplatesView from '../views/TemplatesView.vue';
import VolumesView from '../views/VolumesView.vue';
import NetworksView from '../views/NetworksView.vue';
import SettingsView from '../views/SettingsView.vue';
import CleanupView from '../views/CleanupView.vue';

const routes = [
    { path: '/', redirect: '/dashboard' },
    { path: '/dashboard', name: 'Dashboard', component: DashboardView },
    { path: '/containers', name: 'Containers', component: ContainersView },
    { path: '/images', name: 'Images', component: ImagesView },
    { path: '/volumes', name: 'Volumes', component: VolumesView },
    { path: '/networks', name: 'Networks', component: NetworksView },
    { path: '/templates', name: 'Templates', component: TemplatesView },
    { path: '/settings', name: 'Settings', component: SettingsView },
    { path: '/cleanup', name: 'Cleanup', component: CleanupView },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

export default router;
