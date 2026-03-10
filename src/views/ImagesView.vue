<template>
  <div class="p-6 space-y-4">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold text-white">Images</h2>
        <p class="text-xs text-[var(--color-muted)] mt-1">Manage your Docker images</p>
      </div>
      <div class="flex gap-2">
        <button @click="showPullModal = true" class="cursor-pointer flex items-center gap-1.5 bg-[var(--color-success)]/15 text-[var(--color-success)] text-xs px-3 py-1.5 rounded-lg hover:bg-[var(--color-success)]/25 transition">
          <AppIcon name="pull" :size="13" class="pointer-events-none" /> Pull Image
        </button>
        <button @click="fetchImages" class="cursor-pointer flex items-center gap-1.5 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white text-xs px-3 py-1.5 rounded-lg transition">
          <AppIcon name="refresh" :size="13" class="pointer-events-none" /> Refresh
        </button>
      </div>
    </div>

    <div class="relative">
      <AppIcon name="search" :size="14" class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-muted)]" />
      <input v-model="searchQuery" placeholder="Search images..." class="w-full bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-lg pl-9 pr-3 py-2 text-sm text-white placeholder-[var(--color-muted)] focus:outline-none focus:border-[var(--color-accent)] transition" />
    </div>

    <div v-if="loading" class="flex items-center justify-center py-16">
      <div class="animate-spin w-6 h-6 border-2 border-[var(--color-accent)] border-t-transparent rounded-full"></div>
    </div>

    <div v-else-if="filteredImages.length === 0" class="text-center py-16">
      <AppIcon name="image-empty" :size="48" class="text-[var(--color-border)] mb-3 block mx-auto" />
      <p class="text-[var(--color-muted)] text-sm">No images found</p>
    </div>

    <div v-else class="border border-[var(--color-border)] rounded-xl overflow-hidden">
      <table class="w-full text-sm">
        <thead>
          <tr class="bg-[var(--color-surface-alt)] text-[var(--color-muted)] text-xs">
            <th class="py-2 px-4 text-left font-medium">Repository</th>
            <th class="py-2 px-4 text-left font-medium">Tag</th>
            <th class="py-2 px-4 text-left font-medium">ID</th>
            <th class="py-2 px-4 text-right font-medium">Size</th>
            <th class="py-2 px-4 text-right font-medium">Created</th>
            <th class="py-2 px-4 text-right font-medium w-16"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="image in filteredImages"
            :key="image.Id"
            @contextmenu.prevent="openCtx($event, image)"
            class="border-t border-[var(--color-border)] hover:bg-[var(--color-surface-hover)] transition group"
          >
            <td class="py-2.5 px-4"><span class="text-white font-medium">{{ imageRepo(image) }}</span></td>
            <td class="py-2.5 px-4">
              <span class="text-[10px] bg-[var(--color-accent)]/10 text-[var(--color-accent-hover)] px-1.5 py-0.5 rounded">{{ imageTag(image) }}</span>
            </td>
            <td class="py-2.5 px-4 text-[var(--color-muted)] font-mono text-xs">{{ image.Id?.replace('sha256:', '').slice(0, 12) }}</td>
            <td class="py-2.5 px-4 text-right text-[var(--color-muted)]">{{ formatSize(image.Size) }}</td>
            <td class="py-2.5 px-4 text-right text-[var(--color-muted)]">{{ formatDate(image.Created) }}</td>
            <td class="py-2.5 px-4 text-right">
              <button @click="confirmDelete(image)" class="cursor-pointer p-1 rounded-md text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 opacity-0 group-hover:opacity-100 transition">
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
          <h3 class="text-sm font-semibold text-white mb-3">Image — {{ imageRepo(inspectTarget) }}:{{ imageTag(inspectTarget) }}</h3>
          <pre class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg p-3 text-xs text-gray-300 font-mono whitespace-pre-wrap overflow-auto max-h-96">{{ JSON.stringify(inspectData, null, 2) }}</pre>
          <div class="flex justify-end mt-3">
            <button @click="showInspect = false" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Close</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Pull Image Modal -->
    <Teleport to="body">
      <div v-if="showPullModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" @click.self="showPullModal = false">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-5 w-96 shadow-2xl">
          <h3 class="text-sm font-semibold text-white mb-3">Pull Image</h3>
          <input v-model="pullImageName" placeholder="e.g. nginx:latest, node:20-alpine" class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-sm text-white placeholder-[var(--color-muted)] focus:outline-none focus:border-[var(--color-accent)] mb-3 transition" @keydown.enter="doPull" />
          <div v-if="pullStatus" class="text-xs text-[var(--color-muted)] mb-3 px-1">{{ pullStatus }}</div>
          <div class="flex gap-2 justify-end">
            <button @click="showPullModal = false" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Cancel</button>
            <button @click="doPull" :disabled="pulling" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg bg-[var(--color-accent)] text-white hover:opacity-90 transition disabled:opacity-50">
              {{ pulling ? 'Pulling...' : 'Pull' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Delete Confirmation -->
    <Teleport to="body">
      <div v-if="showDeleteModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" @click.self="showDeleteModal = false">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-5 w-80 shadow-2xl">
          <h3 class="text-sm font-semibold text-white mb-2">Delete Image</h3>
          <p class="text-xs text-[var(--color-muted)] mb-4">
            Delete <span class="text-white font-medium">{{ imageRepo(deleteTarget!) }}:{{ imageTag(deleteTarget!) }}</span>?
          </p>
          <div class="flex gap-2 justify-end">
            <button @click="showDeleteModal = false" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Cancel</button>
            <button @click="doDelete" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg bg-[var(--color-danger)] text-white hover:opacity-90 transition">Delete</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import AppIcon from '../components/AppIcon.vue';
import ContextMenu from '../components/ContextMenu.vue';
import type { MenuItem } from '../components/ContextMenu.vue';

const allImages = ref<any[]>([]);
const loading = ref(true);
const searchQuery = ref('');
const showPullModal = ref(false);
const pullImageName = ref('');
const pulling = ref(false);
const pullStatus = ref('');
const showDeleteModal = ref(false);
const deleteTarget = ref<any>(null);
const showInspect = ref(false);
const inspectData = ref<any>(null);
const inspectTarget = ref<any>(null);

const ctxVisible = ref(false);
const ctxX = ref(0);
const ctxY = ref(0);
const ctxItems = ref<MenuItem[]>([]);

const filteredImages = computed(() => {
  const q = searchQuery.value.toLowerCase();
  if (!q) return allImages.value;
  return allImages.value.filter((img: any) => (img.RepoTags?.[0] || '').toLowerCase().includes(q));
});

function imageRepo(img: any) { return (img?.RepoTags?.[0] || '<none>').split(':')[0]; }
function imageTag(img: any) { return (img?.RepoTags?.[0] || '<none>:<none>').split(':')[1] || 'latest'; }

function formatSize(bytes: number) {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  let i = 0; let size = bytes;
  while (size >= 1024 && i < units.length - 1) { size /= 1024; i++; }
  return `${size.toFixed(1)} ${units[i]}`;
}

function formatDate(timestamp: number) {
  if (!timestamp) return '';
  return new Date(timestamp * 1000).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

function openCtx(e: MouseEvent, image: any) {
  ctxX.value = e.clientX;
  ctxY.value = e.clientY;
  ctxItems.value = [
    { label: 'Inspect', icon: 'inspect', action: () => doInspect(image) },
    { separator: true },
    { label: 'Delete', icon: 'trash', danger: true, action: () => confirmDelete(image) },
  ];
  ctxVisible.value = true;
}

async function doInspect(image: any) {
  try {
    inspectTarget.value = image;
    inspectData.value = await invoke('inspect_image', { id: image.Id });
    showInspect.value = true;
  } catch (e) { console.error(e); }
}

async function fetchImages() {
  loading.value = true;
  try { allImages.value = await invoke<any[]>('list_images'); } catch (e) { console.error(e); }
  finally { loading.value = false; }
}

async function doPull() {
  if (!pullImageName.value.trim()) return;
  pulling.value = true; pullStatus.value = 'Pulling image...';
  let unlisten: (() => void) | undefined;
  try {
    unlisten = await listen<string>('pull-progress', (event) => {
      pullStatus.value = event.payload;
    });
    await invoke('pull_image', { name: pullImageName.value.trim() });
    pullStatus.value = 'Done!'; await fetchImages();
    setTimeout(() => { showPullModal.value = false; pullStatus.value = ''; pullImageName.value = ''; }, 800);
  } catch (e: any) { pullStatus.value = `Error: ${e}`; }
  finally { 
    if (unlisten) unlisten();
    pulling.value = false; 
  }
}

function confirmDelete(image: any) { deleteTarget.value = image; showDeleteModal.value = true; }

async function doDelete() {
  if (!deleteTarget.value) return;
  try { await invoke('remove_image', { id: deleteTarget.value.Id }); await fetchImages(); } catch (e) { console.error(e); }
  showDeleteModal.value = false; deleteTarget.value = null;
}

onMounted(() => fetchImages());
</script>
