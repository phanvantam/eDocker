<template>
  <div class="flex h-screen bg-[var(--color-surface)] text-gray-200 font-sans overflow-hidden">
    <!-- Sidebar -->
    <aside class="w-52 bg-[var(--color-surface-alt)] border-r border-[var(--color-border)] flex flex-col shrink-0">
      <!-- Logo -->
      <div class="px-4 py-3 border-b border-[var(--color-border)]">
        <h1 class="text-base font-bold tracking-tight">
          <span class="text-[var(--color-accent)]">e</span>Docker
        </h1>
        <p class="text-[10px] text-[var(--color-muted)] mt-0.5">Dev Environment Manager</p>
      </div>

      <!-- Nav -->
      <nav class="flex-1 py-2 px-2 space-y-0.5">
        <router-link
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="flex items-center gap-2.5 px-3 py-2 rounded-lg text-[13px] transition-all duration-150"
          :class="$route.path === item.path
            ? 'bg-[var(--color-accent)]/10 text-[var(--color-accent-hover)] font-medium'
            : 'text-gray-400 hover:bg-[var(--color-surface-hover)] hover:text-gray-200'"
        >
          <AppIcon :name="item.icon" :size="16" />
          <span>{{ item.label }}</span>
          <span
            v-if="item.badge !== undefined"
            class="ml-auto text-[10px] bg-[var(--color-surface)] px-1.5 py-0.5 rounded-full text-[var(--color-muted)]"
          >{{ item.badge }}</span>
        </router-link>
      </nav>

      <!-- Docker Status -->
      <div class="px-3 py-2.5 border-t border-[var(--color-border)]">
        <div class="flex items-center gap-2 text-[11px]">
          <span class="w-1.5 h-1.5 rounded-full" :class="dockerConnected ? 'bg-[var(--color-success)]' : 'bg-[var(--color-danger)]'"></span>
          <span class="text-[var(--color-muted)]">{{ dockerConnected ? 'Docker Connected' : 'Disconnected' }}</span>
        </div>
      </div>
    </aside>

    <!-- Main -->
    <main class="flex-1 overflow-y-auto">
      <router-view />
    </main>

    <!-- Connection Error Modal -->
    <Teleport to="body">
      <div v-if="!dockerConnected" class="fixed inset-0 bg-black/80 flex items-center justify-center z-[100] backdrop-blur-md">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-2xl p-8 w-[400px] shadow-2xl text-center">
          <div class="w-16 h-16 bg-[var(--color-danger)]/10 text-[var(--color-danger)] rounded-full flex items-center justify-center mx-auto mb-6">
            <AppIcon name="close" :size="32" />
          </div>
          <h2 class="text-xl font-bold text-white mb-2">Docker Disconnected</h2>
          <p class="text-sm text-[var(--color-muted)] mb-6">
            {{ dockerError || 'Unable to connect to Docker engine. Please ensure Docker Desktop is running.' }}
          </p>
          <button 
            @click="checkDocker" 
            class="w-full flex items-center justify-center gap-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white font-medium py-2.5 rounded-xl transition shadow-lg shadow-[var(--color-accent)]/20"
          >
            <AppIcon name="refresh" :size="16" />
            Check Connection
          </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import AppIcon from './components/AppIcon.vue';

const dockerConnected = ref(true); // Default to true to avoid flicker on first load if possible, or false if you prefer safe
const dockerError = ref('');
let checkInterval: any = null;

const navItems = ref([
  { path: '/dashboard', label: 'Dashboard', icon: 'dashboard', badge: undefined as number | undefined },
  { path: '/containers', label: 'Containers', icon: 'container', badge: undefined as number | undefined },
  { path: '/images', label: 'Images', icon: 'image', badge: undefined as number | undefined },
  { path: '/volumes', label: 'Volumes', icon: 'volume', badge: undefined as number | undefined },
  { path: '/networks', label: 'Networks', icon: 'network', badge: undefined as number | undefined },
  { path: '/templates', label: 'Templates', icon: 'template', badge: undefined as number | undefined },
  { path: '/cleanup', label: 'Cleanup', icon: 'prune', badge: undefined as number | undefined },
  { path: '/settings', label: 'Settings', icon: 'settings', badge: undefined as number | undefined },
]);

async function checkDocker() {
  try {
    await invoke('get_docker_info');
    dockerConnected.value = true;
    dockerError.value = '';
    const containers = await invoke<any[]>('list_containers');
    const images = await invoke<any[]>('list_images');
    navItems.value[1].badge = containers.length;
    navItems.value[2].badge = images.length;
  } catch (e: any) {
    dockerConnected.value = false;
    dockerError.value = e?.toString() || 'Connection failed';
  }
}

onMounted(() => {
  checkDocker();
  checkInterval = setInterval(checkDocker, 15000);
});

onUnmounted(() => {
  if (checkInterval) clearInterval(checkInterval);
});
</script>