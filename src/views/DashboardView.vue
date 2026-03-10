<template>
  <div class="p-6 space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold text-white">Dashboard</h2>
        <p class="text-xs text-[var(--color-muted)] mt-1">System overview and resource usage</p>
      </div>
      <button @click="refreshAll" class="cursor-pointer flex items-center gap-1.5 bg-[var(--color-surface-alt)] hover:bg-[var(--color-surface-hover)] text-white text-xs px-3 py-1.5 rounded-lg border border-[var(--color-border)] transition">
        <AppIcon name="refresh" :size="13" class="pointer-events-none" /> Refresh
      </button>
    </div>

    <!-- Stats Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      <div v-for="stat in stats" :key="stat.label" class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-4 flex items-center gap-4 hover:border-[var(--color-border-hover)] transition group">
        <div class="p-3 rounded-lg bg-[var(--color-surface)] text-[var(--color-accent-hover)] group-hover:scale-110 transition-transform">
          <AppIcon :name="stat.icon" :size="24" />
        </div>
        <div>
          <p class="text-[10px] uppercase tracking-wider text-[var(--color-muted)] font-medium">{{ stat.label }}</p>
          <p class="text-2xl font-bold text-white leading-none mt-1">{{ stat.value }}</p>
          <p class="text-[10px] text-[var(--color-muted)] mt-1.5">{{ stat.subtext }}</p>
        </div>
      </div>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <!-- Docker Info -->
      <div class="lg:col-span-2 bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl overflow-hidden">
        <div class="px-4 py-3 border-b border-[var(--color-border)] bg-[var(--color-surface)]/50 flex items-center gap-2">
          <AppIcon name="docker" :size="16" class="text-[var(--color-accent)]" />
          <h3 class="text-sm font-semibold text-white">Docker Host Information</h3>
        </div>
        <div class="p-4 grid grid-cols-1 md:grid-cols-2 gap-y-4 gap-x-8">
          <div v-for="(val, key) in hostInfo" :key="key" class="flex justify-between border-b border-[var(--color-border)]/50 pb-2">
            <span class="text-xs text-[var(--color-muted)]">{{ key }}</span>
            <span class="text-xs text-gray-200 font-medium">{{ val }}</span>
          </div>
        </div>
      </div>

      <!-- Quick Actions / Status -->
      <div class="space-y-6">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-4">
          <h3 class="text-sm font-semibold text-white mb-4">Quick Links</h3>
          <div class="grid grid-cols-2 gap-2">
            <router-link to="/containers" class="flex flex-col items-center justify-center p-3 rounded-xl bg-[var(--color-surface)] border border-[var(--color-border)] hover:bg-[var(--color-surface-hover)] transition gap-2">
              <AppIcon name="container" :size="20" class="text-[var(--color-success)]" />
              <span class="text-[11px] text-gray-300">Containers</span>
            </router-link>
            <router-link to="/images" class="flex flex-col items-center justify-center p-3 rounded-xl bg-[var(--color-surface)] border border-[var(--color-border)] hover:bg-[var(--color-surface-hover)] transition gap-2">
              <AppIcon name="image" :size="20" class="text-[var(--color-accent)]" />
              <span class="text-[11px] text-gray-300">Images</span>
            </router-link>
            <router-link to="/volumes" class="flex flex-col items-center justify-center p-3 rounded-xl bg-[var(--color-surface)] border border-[var(--color-border)] hover:bg-[var(--color-surface-hover)] transition gap-2">
              <AppIcon name="volume" :size="20" class="text-[var(--color-warning)]" />
              <span class="text-[11px] text-gray-300">Volumes</span>
            </router-link>
            <router-link to="/networks" class="flex flex-col items-center justify-center p-3 rounded-xl bg-[var(--color-surface)] border border-[var(--color-border)] hover:bg-[var(--color-surface-hover)] transition gap-2">
              <AppIcon name="network" :size="20" class="text-[var(--color-primary)]" />
              <span class="text-[11px] text-gray-300">Networks</span>
            </router-link>
          </div>
        </div>

        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-4">
          <div class="flex items-center justify-between mb-4">
             <h3 class="text-sm font-semibold text-white">System Status</h3>
             <span class="text-[10px] px-2 py-0.5 rounded-full bg-[var(--color-success)]/10 text-[var(--color-success)] border border-[var(--color-success)]/20">Active</span>
          </div>
          <div class="space-y-3">
            <div class="flex items-center justify-between text-[11px]">
              <span class="text-[var(--color-muted)]">Cores</span>
              <span class="text-white font-mono">{{ infoRaw?.NCPU || 0 }}</span>
            </div>
            <div class="flex items-center justify-between text-[11px]">
               <span class="text-[var(--color-muted)]">Memory</span>
               <span class="text-white font-mono">{{ formatSize(infoRaw?.MemTotal || 0) }}</span>
            </div>
             <div class="flex items-center justify-between text-[11px]">
               <span class="text-[var(--color-muted)]">Driver</span>
               <span class="text-white font-mono">{{ infoRaw?.Driver || '—' }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import AppIcon from '../components/AppIcon.vue';

const loading = ref(true);
const containers = ref<any[]>([]);
const images = ref<any[]>([]);
const volumes = ref<any[]>([]);
const networks = ref<any[]>([]);
const infoRaw = ref<any>(null);

const stats = computed(() => [
  {
    label: 'Containers',
    value: containers.value.length,
    icon: 'container',
    subtext: `${containers.value.filter(c => c.State === 'running').length} running`
  },
  {
    label: 'Images',
    value: images.value.length,
    icon: 'image',
    subtext: formatSize(images.value.reduce((acc, img) => acc + (img.Size || 0), 0))
  },
  {
    label: 'Volumes',
    value: volumes.value.length,
    icon: 'volume',
    subtext: `${volumes.value.length} total local`
  },
  {
    label: 'Networks',
    value: networks.value.length,
    icon: 'network',
    subtext: `${networks.value.filter(n => n.Driver !== 'null').length} active drivers`
  }
]);

const hostInfo = computed(() => {
  if (!infoRaw.value) return {};
  return {
    'Kernel Version': infoRaw.value.KernelVersion,
    'Operating System': infoRaw.value.OperatingSystem,
    'OS Version': infoRaw.value.OSVersion,
    'Architecture': infoRaw.value.Architecture,
    'Docker Version': infoRaw.value.ServerVersion,
    'Root Dir': infoRaw.value.DockerRootDir,
    'Storage Driver': infoRaw.value.Driver,
    'Logging Driver': infoRaw.value.LoggingDriver,
  };
});

function formatSize(bytes: number) {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  let i = 0; let size = bytes;
  while (size >= 1024 && i < units.length - 1) { size /= 1024; i++; }
  return `${size.toFixed(1)} ${units[i]}`;
}

async function refreshAll() {
  loading.value = true;
  try {
    const [c, img, v, n, info] = await Promise.all([
      invoke<any[]>('list_containers'),
      invoke<any[]>('list_images'),
      invoke<any>('list_volumes'),
      invoke<any[]>('list_networks'),
      invoke<any>('get_docker_info')
    ]);
    containers.value = c;
    images.value = img;
    volumes.value = v?.Volumes || [];
    networks.value = n;
    infoRaw.value = info;
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
}

onMounted(() => refreshAll());
</script>
