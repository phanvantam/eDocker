<template>
  <div class="flex h-full">
    <!-- Container List -->
    <div class="flex-1 p-6 space-y-4 overflow-y-auto">
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-xl font-semibold text-white">Containers</h2>
          <p class="text-xs text-[var(--color-muted)] mt-1">Grouped by docker-compose project</p>
        </div>
        <button @click="fetchContainers" class="cursor-pointer flex items-center gap-1.5 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white text-xs px-3 py-1.5 rounded-lg transition">
          <AppIcon name="refresh" :size="13" class="pointer-events-none" /> Refresh
        </button>
      </div>

      <div v-if="loading" class="flex items-center justify-center py-16">
        <div class="animate-spin w-6 h-6 border-2 border-[var(--color-accent)] border-t-transparent rounded-full"></div>
      </div>

      <div v-else-if="Object.keys(groupedContainers).length === 0" class="text-center py-16">
        <AppIcon name="box-empty" :size="48" class="text-[var(--color-border)] mb-3 block mx-auto" />
        <p class="text-[var(--color-muted)] text-sm">No containers found</p>
      </div>

      <div v-else class="space-y-3">
        <div v-for="(containers, group) in groupedContainers" :key="group" class="rounded-xl border border-[var(--color-border)] overflow-hidden">
          <button @click="toggleGroup(group as string)" class="w-full flex items-center gap-2.5 px-4 py-2 bg-[var(--color-surface-alt)] hover:bg-[var(--color-surface-hover)] transition text-left group">
            <AppIcon name="chevron" :size="12" class="transition-transform" :class="expandedGroups.has(group as string) ? 'rotate-90' : ''" />
            <span class="text-sm font-medium text-white">{{ group }}</span>
            <span class="text-[10px] text-[var(--color-muted)] bg-[var(--color-surface)] px-1.5 py-0.5 rounded-full">{{ containers.length }}</span>
            <div class="ml-auto flex items-center gap-1.5 z-10 relative" @click.stop>
              <span v-if="countByState(containers, 'running')" class="flex items-center gap-1 text-[10px] text-[var(--color-success)] cursor-default">
                <span class="w-1.5 h-1.5 rounded-full bg-[var(--color-success)]"></span>{{ countByState(containers, 'running') }}
              </span>
              <span v-if="countByState(containers, 'exited')" class="flex items-center gap-1 text-[10px] text-[var(--color-danger)] cursor-default">
                <span class="w-1.5 h-1.5 rounded-full bg-[var(--color-danger)]"></span>{{ countByState(containers, 'exited') }}
              </span>
              <button @click.self="confirmRemoveGroup(group as string, containers)" class="cursor-pointer relative z-20 p-1.5 rounded-md text-[var(--color-danger)] opacity-0 group-hover:opacity-100 hover:bg-[var(--color-danger)]/10 transition ml-2" title="Delete Group">
                <AppIcon name="trash" :size="13" class="pointer-events-none" />
              </button>
            </div>
          </button>

          <div v-show="expandedGroups.has(group as string)">
            <div
              v-for="container in containers"
              :key="container.Id"
              @contextmenu.prevent="openCtx($event, container)"
              @click="selectContainer(container)"
              class="flex items-center gap-2.5 px-4 py-2 border-t border-[var(--color-border)] hover:bg-[var(--color-surface-hover)] transition group cursor-pointer"
              :class="{ 'bg-[var(--color-accent)]/5 border-l-2 border-l-[var(--color-accent)]': selectedContainer?.Id === container.Id }"
            >
              <div class="w-3 text-[var(--color-border)] text-xs pl-0.5">└</div>
              <span class="w-2 h-2 rounded-full shrink-0" :class="stateColor(container.State)"></span>
              <div class="flex-1 min-w-0">
                <p class="text-sm font-medium text-white truncate">{{ containerName(container) }}</p>
                <p class="text-[10px] text-[var(--color-muted)] truncate">{{ container.Image }} · {{ container.Status }}</p>
              </div>
              <div v-if="container.Ports?.length" class="hidden lg:flex gap-1 shrink-0">
                <span v-for="port in container.Ports.slice(0, 2)" :key="port.PublicPort" class="text-[9px] bg-[var(--color-surface)] text-[var(--color-muted)] px-1.5 py-0.5 rounded">{{ port.PublicPort || port.PrivatePort }}:{{ port.PrivatePort }}</span>
              </div>
              <div class="flex items-center gap-0.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                <button v-if="container.State !== 'running'" @click.stop="action('start_container', container.Id)" class="cursor-pointer p-1 rounded-md text-[var(--color-success)] hover:bg-[var(--color-success)]/10 transition" title="Start">
                  <AppIcon name="play" :size="12" class="pointer-events-none" />
                </button>
                <button v-if="container.State === 'running'" @click.stop="action('stop_container', container.Id)" class="cursor-pointer p-1 rounded-md text-[var(--color-warning)] hover:bg-[var(--color-warning)]/10 transition" title="Stop">
                  <AppIcon name="stop" :size="12" class="pointer-events-none" />
                </button>
                <button @click.stop="action('restart_container', container.Id)" class="cursor-pointer p-1 rounded-md text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10 transition" title="Restart">
                  <AppIcon name="restart" :size="12" class="pointer-events-none" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Detail Panel -->
    <div v-if="selectedContainer" class="w-[400px] border-l border-[var(--color-border)] flex flex-col bg-[var(--color-surface-alt)] shrink-0">
      <div class="flex items-center justify-between px-4 py-2.5 border-b border-[var(--color-border)]">
        <div class="min-w-0">
          <p class="text-sm font-medium text-white truncate">{{ containerName(selectedContainer) }}</p>
          <p class="text-[10px] text-[var(--color-muted)]">{{ selectedContainer.Image }}</p>
        </div>
        <button @click="selectedContainer = null" class="cursor-pointer p-1 text-[var(--color-muted)] hover:text-white transition">
           <AppIcon name="close" :size="14" class="pointer-events-none" />
        </button>
      </div>
      <!-- Tabs -->
      <div class="flex border-b border-[var(--color-border)]">
        <button
          v-for="tab in detailTabs"
          :key="tab.id"
          @click="activeTab = tab.id"
          class="cursor-pointer flex items-center gap-1.5 px-3 py-2 text-[11px] transition border-b-2"
          :class="activeTab === tab.id ? 'text-[var(--color-accent-hover)] border-[var(--color-accent)]' : 'text-[var(--color-muted)] border-transparent hover:text-gray-300'"
        >
          <AppIcon :name="tab.icon" :size="13" class="pointer-events-none" />
          {{ tab.label }}
        </button>
      </div>
      <!-- Tab Content -->
      <div class="flex-1 overflow-hidden">
        <ContainerLogs v-if="activeTab === 'logs'" :containerId="selectedContainer.Id" :name="containerName(selectedContainer)" />
        <ContainerInfo v-else-if="activeTab === 'info'" :containerId="selectedContainer.Id" :name="containerName(selectedContainer)" />
      </div>
    </div>

    <!-- Context Menu -->
    <ContextMenu :visible="ctxVisible" :x="ctxX" :y="ctxY" :items="ctxItems" @close="ctxVisible = false" />

    <!-- Delete Confirmation Modal -->
    <Teleport to="body">
      <div v-if="showDeleteModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" @click.self="showDeleteModal = false">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-5 w-80 shadow-2xl">
          <h3 class="text-sm font-semibold text-white mb-2">Delete Container</h3>
          <p class="text-xs text-[var(--color-muted)] mb-4">
            Delete <span class="text-white font-medium">{{ containerName(deleteTarget!) }}</span>? This will force-remove the container.
          </p>
          <div class="flex gap-2 justify-end">
            <button @click="showDeleteModal = false" class="px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Cancel</button>
            <button @click="doRemove" class="px-3 py-1.5 text-xs rounded-lg bg-[var(--color-danger)] text-white hover:opacity-90 transition">Delete</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Group Delete Confirmation Modal -->
    <Teleport to="body">
      <div v-if="showGroupDeleteModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" @click.self="showGroupDeleteModal = false">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-5 w-80 shadow-2xl">
          <h3 class="text-sm font-semibold text-white mb-2">Delete Group</h3>
          <p class="text-xs text-[var(--color-muted)] mb-4">
            Delete <span class="text-white font-medium">{{ groupDeleteTarget }}</span>? This will remove <span class="text-white font-medium">{{ groupContainersToDelete.length }}</span> containers.
          </p>
          <div class="flex gap-2 justify-end">
            <button @click="showGroupDeleteModal = false" class="px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Cancel</button>
            <button @click="doRemoveGroup" class="px-3 py-1.5 text-xs rounded-lg bg-[var(--color-danger)] text-white hover:opacity-90 transition">Delete All</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import AppIcon from '../components/AppIcon.vue';
import ContextMenu from '../components/ContextMenu.vue';
import ContainerLogs from '../components/ContainerLogs.vue';
import ContainerInfo from '../components/ContainerInfo.vue';
import type { MenuItem } from '../components/ContextMenu.vue';

const allContainers = ref<any[]>([]);
const loading = ref(true);
const expandedGroups = ref(new Set<string>());
const selectedContainer = ref<any>(null);
const activeTab = ref('logs');
const showDeleteModal = ref(false);
const deleteTarget = ref<any>(null);
const showGroupDeleteModal = ref(false);
const groupDeleteTarget = ref('');
const groupContainersToDelete = ref<any[]>([]);

const ctxVisible = ref(false);
const ctxX = ref(0);
const ctxY = ref(0);
const ctxItems = ref<MenuItem[]>([]);

const detailTabs = [
  { id: 'logs', label: 'Logs', icon: 'logs' },
  { id: 'info', label: 'Info', icon: 'info' },
];

const groupedContainers = computed(() => {
  const groups: Record<string, any[]> = {};
  for (const c of allContainers.value) {
    const project = c.Labels?.['com.docker.compose.project'] || '_standalone';
    if (!groups[project]) groups[project] = [];
    groups[project].push(c);
  }
  for (const key of Object.keys(groups)) {
    groups[key].sort((a: any, b: any) => (a.State === 'running' ? -1 : 1) - (b.State === 'running' ? -1 : 1));
  }
  return groups;
});

function containerName(c: any) { return c.Names?.[0]?.replace(/^\//, '') || c.Id?.slice(0, 12); }
function stateColor(state: string) {
  if (state === 'running') return 'bg-[var(--color-success)]';
  if (state === 'paused') return 'bg-[var(--color-warning)]';
  return 'bg-[var(--color-danger)]';
}
function countByState(containers: any[], state: string) { return containers.filter((c: any) => c.State === state).length; }

function toggleGroup(group: string) {
  expandedGroups.value.has(group) ? expandedGroups.value.delete(group) : expandedGroups.value.add(group);
}

function selectContainer(c: any) {
  selectedContainer.value = c;
  activeTab.value = 'logs';
}

function openCtx(e: MouseEvent, container: any) {
  ctxX.value = e.clientX;
  ctxY.value = e.clientY;
  const running = container.State === 'running';
  ctxItems.value = [
    ...(running ? [] : [{ label: 'Start', icon: 'play', action: () => action('start_container', container.Id) }]),
    ...(running ? [{ label: 'Stop', icon: 'stop', action: () => action('stop_container', container.Id) }] : []),
    { label: 'Restart', icon: 'restart', action: () => action('restart_container', container.Id) },
    ...(running ? [{ label: 'Pause', icon: 'pause', action: () => action('pause_container', container.Id) }] : []),
    ...(container.State === 'paused' ? [{ label: 'Unpause', icon: 'play', action: () => action('unpause_container', container.Id) }] : []),
    { separator: true },
    { label: 'View Logs', icon: 'logs', action: () => { selectContainer(container); activeTab.value = 'logs'; } },
    { label: 'Info', icon: 'info', action: () => { selectContainer(container); activeTab.value = 'info'; } },
    { separator: true },
    { label: 'Delete', icon: 'trash', danger: true, action: () => confirmRemove(container) },
  ];
  ctxVisible.value = true;
}

async function fetchContainers() {
  loading.value = true;
  try {
    allContainers.value = await invoke<any[]>('list_containers');
    for (const key of Object.keys(groupedContainers.value)) {
      expandedGroups.value.add(key);
    }
  } catch (e) { console.error(e); }
  finally { loading.value = false; }
}

async function action(cmd: string, id: string) {
  try { await invoke(cmd, { id }); await fetchContainers(); } catch (e) { console.error(e); }
}

function confirmRemove(container: any) {
  deleteTarget.value = container;
  showDeleteModal.value = true;
}

async function doRemove() {
  if (!deleteTarget.value) return;
  await action('remove_container', deleteTarget.value.Id);
  if (selectedContainer.value?.Id === deleteTarget.value.Id) selectedContainer.value = null;
  showDeleteModal.value = false;
  deleteTarget.value = null;
}

function confirmRemoveGroup(group: string, containers: any[]) {
  groupDeleteTarget.value = group;
  groupContainersToDelete.value = containers;
  showGroupDeleteModal.value = true;
}

async function doRemoveGroup() {
  if (!groupDeleteTarget.value) return;
  loading.value = true;
  try {
    for (const container of groupContainersToDelete.value) {
      await invoke('remove_container', { id: container.Id });
      if (selectedContainer.value?.Id === container.Id) selectedContainer.value = null;
    }
    await fetchContainers();
  } catch (e) {
    console.error(e);
  } finally {
    showGroupDeleteModal.value = false;
    groupDeleteTarget.value = '';
    groupContainersToDelete.value = [];
    loading.value = false;
  }
}

onMounted(() => fetchContainers());
</script>
