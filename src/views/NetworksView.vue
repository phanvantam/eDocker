<template>
  <div class="p-6 space-y-4">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold text-white">Networks</h2>
        <p class="text-xs text-[var(--color-muted)] mt-1">Manage Docker networks</p>
      </div>
      <div class="flex gap-2">
        <button @click="showCreateModal = true" class="cursor-pointer flex items-center gap-1.5 bg-[var(--color-success)]/15 text-[var(--color-success)] text-xs px-3 py-1.5 rounded-lg hover:bg-[var(--color-success)]/25 transition">
          <AppIcon name="plus" :size="13" class="pointer-events-none" /> Create
        </button>
        <button @click="fetchNetworks" class="cursor-pointer flex items-center gap-1.5 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white text-xs px-3 py-1.5 rounded-lg transition">
          <AppIcon name="refresh" :size="13" class="pointer-events-none" /> Refresh
        </button>
      </div>
    </div>

    <div v-if="loading" class="flex items-center justify-center py-16">
      <div class="animate-spin w-6 h-6 border-2 border-[var(--color-accent)] border-t-transparent rounded-full"></div>
    </div>

    <div v-else-if="networks.length === 0" class="text-center py-16">
      <AppIcon name="network-empty" :size="48" class="text-[var(--color-border)] mb-3 block mx-auto" />
      <p class="text-[var(--color-muted)] text-sm">No networks found</p>
    </div>

    <div v-else class="border border-[var(--color-border)] rounded-xl overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="bg-[var(--color-surface-alt)] text-[var(--color-muted)] text-xs">
            <th class="py-2 px-4 text-left font-medium">Name</th>
            <th class="py-2 px-4 text-left font-medium">Driver</th>
            <th class="py-2 px-4 text-left font-medium">Scope</th>
            <th class="py-2 px-4 text-left font-medium">Subnet</th>
            <th class="py-2 px-4 text-right font-medium w-16"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="net in networks"
            :key="net.Id"
            @contextmenu.prevent="openCtx($event, net)"
            class="border-t border-[var(--color-border)] hover:bg-[var(--color-surface-hover)] transition group"
          >
            <td class="py-2.5 px-4 text-white font-medium">{{ net.Name }}</td>
            <td class="py-2.5 px-4">
              <span class="text-[10px] bg-[var(--color-accent)]/10 text-[var(--color-accent-hover)] px-1.5 py-0.5 rounded">{{ net.Driver }}</span>
            </td>
            <td class="py-2.5 px-4 text-[var(--color-muted)]">{{ net.Scope }}</td>
            <td class="py-2.5 px-4 text-[var(--color-muted)] font-mono text-xs">{{ subnet(net) }}</td>
            <td class="py-2.5 px-4 text-right">
              <button
                v-if="!isBuiltIn(net)"
                @click="confirmDelete(net)"
                class="cursor-pointer p-1 rounded-md text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 opacity-0 group-hover:opacity-100 transition"
              >
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
          <h3 class="text-sm font-semibold text-white mb-3">Network — {{ inspectData?.Name }}</h3>
          <div v-if="inspectData?.Containers && Object.keys(inspectData.Containers).length" class="mb-3">
            <h4 class="text-[10px] uppercase text-[var(--color-muted)] mb-1 font-medium">Connected Containers</h4>
            <div class="bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] px-3 py-2 space-y-1">
              <div v-for="(c, id) in inspectData.Containers" :key="id" class="flex justify-between text-[10px]">
                <span class="text-gray-300">{{ c.Name }}</span>
                <span class="text-[var(--color-muted)] font-mono">{{ c.IPv4Address }}</span>
              </div>
            </div>
          </div>
          <pre class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg p-3 text-xs text-gray-300 font-mono whitespace-pre-wrap overflow-auto max-h-72">{{ JSON.stringify(inspectData, null, 2) }}</pre>
          <div class="flex justify-end mt-3">
            <button @click="showInspect = false" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Close</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Create Modal -->
    <Teleport to="body">
      <div v-if="showCreateModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" @click.self="showCreateModal = false">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-5 w-96 shadow-2xl">
          <h3 class="text-sm font-semibold text-white mb-3">Create Network</h3>
          <div class="space-y-3">
            <div>
              <label class="text-[10px] text-[var(--color-muted)] uppercase tracking-wider mb-1 block">Name</label>
              <input v-model="createName" placeholder="my-network" class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-sm text-white placeholder-[var(--color-muted)] focus:outline-none focus:border-[var(--color-accent)] transition" />
            </div>
            <div>
              <label class="text-[10px] text-[var(--color-muted)] uppercase tracking-wider mb-1 block">Driver</label>
              <select v-model="createDriver" class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-[var(--color-accent)] transition">
                <option value="bridge">bridge</option>
                <option value="host">host</option>
                <option value="overlay">overlay</option>
                <option value="macvlan">macvlan</option>
                <option value="none">none</option>
              </select>
            </div>
          </div>
          <div class="flex gap-2 justify-end mt-4">
            <button @click="showCreateModal = false" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Cancel</button>
            <button @click="doCreate" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg bg-[var(--color-accent)] text-white hover:opacity-90 transition">Create</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Delete Confirmation -->
    <Teleport to="body">
      <div v-if="showDeleteModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" @click.self="showDeleteModal = false">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-5 w-80 shadow-2xl">
          <h3 class="text-sm font-semibold text-white mb-2">Delete Network</h3>
          <p class="text-xs text-[var(--color-muted)] mb-4">Delete network "{{ deleteTargetName }}"?</p>
          <div class="flex gap-2 justify-end">
            <button @click="showDeleteModal = false" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Cancel</button>
            <button @click="doDeleteNetwork" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg bg-[var(--color-danger)] text-white hover:opacity-90 transition">Delete</button>
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

const networks = ref<any[]>([]);
const loading = ref(true);
const ctxVisible = ref(false);
const ctxX = ref(0);
const ctxY = ref(0);
const ctxItems = ref<MenuItem[]>([]);
const showInspect = ref(false);
const inspectData = ref<any>(null);
const showCreateModal = ref(false);
const createName = ref('');
const createDriver = ref('bridge');
const showDeleteModal = ref(false);
const deleteTargetName = ref('');
const deleteTargetId = ref('');

const builtIn = ['bridge', 'host', 'none'];

function isBuiltIn(net: any) { return builtIn.includes(net.Name); }
function subnet(net: any) { return net.IPAM?.Config?.[0]?.Subnet || '—'; }

async function fetchNetworks() {
  loading.value = true;
  try {
    networks.value = await invoke<any[]>('list_networks');
  } catch (e) { console.error(e); }
  finally { loading.value = false; }
}

function openCtx(e: MouseEvent, net: any) {
  ctxX.value = e.clientX;
  ctxY.value = e.clientY;
  const items: MenuItem[] = [
    { label: 'Inspect', icon: 'inspect', action: () => doInspect(net.Id) },
  ];
  if (!isBuiltIn(net)) {
    items.push({ separator: true });
    items.push({ label: 'Delete', icon: 'trash', danger: true, action: () => confirmDelete(net) });
  }
  ctxItems.value = items;
  ctxVisible.value = true;
}

async function doInspect(id: string) {
  try {
    inspectData.value = await invoke('inspect_network', { id });
    showInspect.value = true;
  } catch (e) { console.error(e); }
}

function confirmDelete(net: any) {
  deleteTargetName.value = net.Name;
  deleteTargetId.value = net.Id;
  showDeleteModal.value = true;
}

async function doDeleteNetwork() {
  try {
    await invoke('remove_network', { id: deleteTargetId.value });
    await fetchNetworks();
  } catch (e) { console.error(e); }
  showDeleteModal.value = false;
}

async function doCreate() {
  if (!createName.value.trim()) return;
  try {
    await invoke('create_network', { name: createName.value.trim(), driver: createDriver.value });
    createName.value = '';
    showCreateModal.value = false;
    await fetchNetworks();
  } catch (e) { console.error(e); }
}

onMounted(() => fetchNetworks());
</script>
