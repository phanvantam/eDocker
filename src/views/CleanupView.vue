<template>
  <div class="p-6 space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold text-white">Docker Cleanup</h2>
        <p class="text-xs text-[var(--color-muted)] mt-1">Monitor disk usage and reclaim space by pruning unused resources</p>
      </div>
      <button 
        @click="confirmPrune('all')" 
        class="cursor-pointer flex items-center gap-2 bg-[var(--color-danger)] hover:bg-[var(--color-danger-hover)] text-white text-xs px-4 py-2 rounded-lg transition shadow-lg shadow-[var(--color-danger)]/20"
      >
        <AppIcon name="prune" :size="14" class="pointer-events-none" />
        Prune System
      </button>
    </div>

    <!-- Usage Grid -->
    <div v-if="loading" class="flex items-center justify-center p-20">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-[var(--color-accent)]"></div>
    </div>
    
    <div v-else-if="usageData.length" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      <div v-for="item in usageData" :key="item.Type" class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-4 flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between mb-3">
             <div class="flex items-center gap-2">
               <AppIcon :name="getIcon(item.Type)" :size="16" class="text-[var(--color-accent)]" />
               <span class="text-xs font-semibold text-white uppercase tracking-wider">{{ item.Type }}</span>
             </div>
             <span class="text-[10px] bg-[var(--color-surface)] px-1.5 py-0.5 rounded text-[var(--color-muted)]">{{ item.TotalCount }} total</span>
          </div>
          <div class="space-y-1">
            <div class="flex justify-between text-[11px]">
              <span class="text-[var(--color-muted)]">Total Size</span>
              <span class="text-white font-medium">{{ item.Size }}</span>
            </div>
            <div class="flex justify-between text-[11px]">
              <span class="text-[var(--color-muted)]">Reclaimable</span>
              <span class="text-[var(--color-success)] font-bold">{{ item.Reclaimable }}</span>
            </div>
          </div>
        </div>
        
        <button 
          @click="confirmPrune(getTarget(item.Type))"
          class="cursor-pointer mt-4 w-full py-1.5 border border-[var(--color-border)] hover:bg-[var(--color-surface-hover)] text-[10px] text-[var(--color-muted)] hover:text-white rounded-lg transition"
        >
          Prune {{ item.Type }}
        </button>
      </div>
    </div>

    <!-- Output Console -->
    <div v-if="pruneOutput" class="bg-black/40 border border-[var(--color-border)] rounded-xl overflow-hidden">
      <div class="px-4 py-2 border-b border-[var(--color-border)] bg-[var(--color-surface)]/50 flex items-center justify-between">
        <span class="text-[10px] font-semibold text-[var(--color-muted)] uppercase tracking-widest">Execution Result</span>
        <button @click="pruneOutput = ''" class="cursor-pointer text-[var(--color-muted)] hover:text-white">
          <AppIcon name="close" :size="12" class="pointer-events-none" />
        </button>
      </div>
      <pre class="p-4 text-[10px] font-mono text-gray-400 overflow-x-auto whitespace-pre-wrap max-h-60 leading-relaxed">{{ pruneOutput }}</pre>
    </div>

    <!-- Helper Info -->
    <div class="bg-[var(--color-accent)]/5 border border-[var(--color-accent)]/20 p-4 rounded-xl flex gap-3">
      <AppIcon name="info" :size="16" class="text-[var(--color-accent)] shrink-0 mt-0.5" />
      <div class="space-y-1">
        <p class="text-xs font-medium text-white">Understanding Pruning</p>
        <p class="text-[11px] text-[var(--color-muted)] leading-relaxed">
          The <strong>Prune System</strong> command removes all unused containers, networks, images (both dangling and unreferenced), and optionally, volumes. 
          Use individual prune buttons to target specific resource types. This operation is irreversible.
        </p>
      </div>
    </div>

    <!-- Confirmation Modal -->
    <Teleport to="body">
      <div v-if="showModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-[110] backdrop-blur-sm">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-2xl p-6 w-[360px] shadow-2xl">
          <div class="w-12 h-12 bg-[var(--color-danger)]/10 text-[var(--color-danger)] rounded-full flex items-center justify-center mx-auto mb-4">
            <AppIcon name="prune" :size="24" />
          </div>
          <h3 class="text-center font-bold text-white mb-2">Confirm Action</h3>
          <p class="text-center text-xs text-[var(--color-muted)] mb-6 leading-relaxed">
            Are you sure you want to prune {{ modalTargetName }}? This will delete all unused resources and cannot be undone.
          </p>
          <div class="flex gap-3">
            <button @click="showModal = false" class="cursor-pointer flex-1 py-2 rounded-xl text-xs font-medium text-gray-400 hover:bg-[var(--color-surface-hover)] hover:text-white transition">Cancel</button>
            <button @click="runPrune" class="cursor-pointer flex-1 py-2 rounded-xl text-xs font-medium bg-[var(--color-danger)] hover:bg-[var(--color-danger-hover)] text-white transition">Confirm Prune</button>
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

interface UsageItem {
  Type: string;
  TotalCount: string;
  Active: string;
  Size: string;
  Reclaimable: string;
}

const loading = ref(false);
const usageData = ref<UsageItem[]>([]);
const pruneOutput = ref('');
const showModal = ref(false);
const pendingTarget = ref('');
const modalTargetName = ref('');

async function fetchUsage() {
  loading.value = true;
  try {
    usageData.value = await invoke('get_system_df');
  } catch (e: any) {
    console.error('Failed to fetch docker usage:', e);
  } finally {
    loading.value = false;
  }
}

function getIcon(type: string) {
  switch (type.toLowerCase()) {
    case 'images': return 'image';
    case 'containers': return 'container';
    case 'local volumes': return 'volume';
    case 'build cache': return 'refresh';
    default: return 'info';
  }
}

function getTarget(type: string) {
  switch (type.toLowerCase()) {
    case 'images': return 'images';
    case 'containers': return 'containers';
    case 'local volumes': return 'volumes';
    case 'build cache': return 'build_cache';
    default: return '';
  }
}

function confirmPrune(target: string) {
  pendingTarget.value = target;
  modalTargetName.value = target === 'all' ? 'Entire System' : target.replace('_', ' ');
  showModal.value = true;
}

async function runPrune() {
  showModal.value = false;
  loading.value = true;
  pruneOutput.value = '';
  try {
    const result = await invoke<string>('execute_prune', { target: pendingTarget.value });
    pruneOutput.value = result || 'Cleanup completed successfully.';
    await fetchUsage();
  } catch (e: any) {
    pruneOutput.value = `Error during cleanup: ${e}`;
  } finally {
    loading.value = false;
  }
}

onMounted(() => fetchUsage());
</script>
