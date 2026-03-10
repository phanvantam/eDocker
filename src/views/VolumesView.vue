<template>
  <div class="p-6 space-y-4">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold text-white">Volumes</h2>
        <p class="text-xs text-[var(--color-muted)] mt-1">Manage Docker volumes</p>
      </div>
      <div class="flex gap-2">
        <button @click="confirmPrune" class="cursor-pointer flex items-center gap-1.5 bg-[var(--color-warning)]/15 text-[var(--color-warning)] text-xs px-3 py-1.5 rounded-lg hover:bg-[var(--color-warning)]/25 transition">
          <AppIcon name="prune" :size="13" class="pointer-events-none" /> Prune Unused
        </button>
        <button @click="fetchVolumes" class="cursor-pointer flex items-center gap-1.5 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white text-xs px-3 py-1.5 rounded-lg transition">
          <AppIcon name="refresh" :size="13" class="pointer-events-none" /> Refresh
        </button>
      </div>
    </div>

    <div v-if="loading" class="flex items-center justify-center py-16">
      <div class="animate-spin w-6 h-6 border-2 border-[var(--color-accent)] border-t-transparent rounded-full"></div>
    </div>

    <div v-else-if="volumes.length === 0" class="text-center py-16">
      <AppIcon name="disk-empty" :size="48" class="text-[var(--color-border)] mb-3 block mx-auto" />
      <p class="text-[var(--color-muted)] text-sm">No volumes found</p>
    </div>

    <div v-else class="border border-[var(--color-border)] rounded-xl overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="bg-[var(--color-surface-alt)] text-[var(--color-muted)] text-xs">
            <th class="py-2 px-4 text-left font-medium">Name</th>
            <th class="py-2 px-4 text-left font-medium">Driver</th>
            <th class="py-2 px-4 text-left font-medium">Mount Point</th>
            <th class="py-2 px-4 text-right font-medium w-16"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="vol in volumes"
            :key="vol.Name"
            @contextmenu.prevent="openCtx($event, vol)"
            class="border-t border-[var(--color-border)] hover:bg-[var(--color-surface-hover)] transition group"
          >
            <td class="py-2.5 px-4 text-white font-medium font-mono text-xs">{{ vol.Name?.slice(0, 24) }}{{ vol.Name?.length > 24 ? '…' : '' }}</td>
            <td class="py-2.5 px-4 text-[var(--color-muted)]">{{ vol.Driver }}</td>
            <td class="py-2.5 px-4 text-[var(--color-muted)] font-mono text-xs truncate max-w-xs">{{ vol.Mountpoint }}</td>
            <td class="py-2.5 px-4 text-right">
              <button @click="confirmDelete(vol)" class="cursor-pointer p-1 rounded-md text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 opacity-0 group-hover:opacity-100 transition">
                <AppIcon name="trash" :size="14" class="pointer-events-none" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Context Menu -->
    <ContextMenu :visible="ctxVisible" :x="ctxX" :y="ctxY" :items="ctxItems" @close="ctxVisible = false" />

    <!-- Inspect Modal -->
    <Teleport to="body">
      <div v-if="showInspect" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" @click.self="showInspect = false">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-5 w-[520px] max-h-[80vh] overflow-y-auto shadow-2xl">
          <h3 class="text-sm font-semibold text-white mb-3">Volume — {{ inspectData?.Name }}</h3>
          <pre class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg p-3 text-xs text-gray-300 font-mono whitespace-pre-wrap overflow-auto max-h-96">{{ JSON.stringify(inspectData, null, 2) }}</pre>
          <div class="flex justify-end mt-3">
            <button @click="showInspect = false" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Close</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Delete Confirmation -->
    <Teleport to="body">
      <div v-if="showDeleteModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" @click.self="showDeleteModal = false">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-5 w-80 shadow-2xl">
          <h3 class="text-sm font-semibold text-white mb-2">{{ pruneMode ? 'Prune Volumes' : 'Delete Volume' }}</h3>
          <p class="text-xs text-[var(--color-muted)] mb-4">
            {{ pruneMode ? 'Remove all unused volumes? This cannot be undone.' : `Delete volume "${deleteTargetName}"?` }}
          </p>
          <div class="flex gap-2 justify-end">
            <button @click="showDeleteModal = false" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Cancel</button>
            <button @click="doDelete" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg bg-[var(--color-danger)] text-white hover:opacity-90 transition">
              {{ pruneMode ? 'Prune' : 'Delete' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import AppIcon from '../components/AppIcon.vue';
import ContextMenu from '../components/ContextMenu.vue';
import type { MenuItem } from '../components/ContextMenu.vue';

const volumes = ref<any[]>([]);
const loading = ref(true);
const showDeleteModal = ref(false);
const deleteTargetName = ref('');
const pruneMode = ref(false);
const showInspect = ref(false);
const inspectData = ref<any>(null);

const ctxVisible = ref(false);
const ctxX = ref(0);
const ctxY = ref(0);
const ctxItems = ref<MenuItem[]>([]);

async function fetchVolumes() {
  loading.value = true;
  try {
    const data = await invoke<any>('list_volumes');
    volumes.value = data?.Volumes || [];
  } catch (e) { console.error(e); }
  finally { loading.value = false; }
}

function openCtx(e: MouseEvent, vol: any) {
  ctxX.value = e.clientX;
  ctxY.value = e.clientY;
  ctxItems.value = [
    { label: 'Inspect', icon: 'inspect', action: () => doInspect(vol.Name) },
    { separator: true },
    { label: 'Delete', icon: 'trash', danger: true, action: () => confirmDelete(vol) },
  ];
  ctxVisible.value = true;
}

async function doInspect(name: string) {
  try {
    inspectData.value = await invoke('inspect_volume', { name });
    showInspect.value = true;
  } catch (e) { console.error(e); }
}

function confirmDelete(vol: any) {
  deleteTargetName.value = vol.Name;
  pruneMode.value = false;
  showDeleteModal.value = true;
}

function confirmPrune() {
  pruneMode.value = true;
  showDeleteModal.value = true;
}

async function doDelete() {
  try {
    if (pruneMode.value) {
      await invoke('prune_volumes');
    } else {
      await invoke('remove_volume', { name: deleteTargetName.value });
    }
    await fetchVolumes();
  } catch (e) { console.error(e); }
  showDeleteModal.value = false;
}

onMounted(() => fetchVolumes());
</script>
