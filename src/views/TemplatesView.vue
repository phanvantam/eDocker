<template>
  <div class="p-6 space-y-4">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold text-white">Templates</h2>
        <p class="text-xs text-[var(--color-muted)] mt-1">Pre-built docker-compose stacks for quick setup</p>
      </div>
      <button @click="showCreateModal = true" class="cursor-pointer flex items-center gap-1.5 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white text-xs px-3 py-1.5 rounded-lg transition">
        + New Template
      </button>
    </div>

    <!-- Template Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <div
        v-for="tpl in templates"
        :key="tpl.name"
        class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-4 hover:border-[var(--color-border-hover)] transition group"
      >
        <div class="flex items-start justify-between mb-3">
          <div>
            <h3 class="text-sm font-semibold text-white flex items-center gap-2">
              <AppIcon :name="tpl.icon || 'docker'" :size="20" class="text-[var(--color-accent)]" />
              {{ tpl.name }}
            </h3>
            <p class="text-[10px] text-[var(--color-muted)] mt-1">{{ tpl.description }}</p>
          </div>
          <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button @click="editTemplate(tpl)" class="cursor-pointer p-1 rounded-md text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10 transition" title="Edit">
              <AppIcon name="edit" :size="12" class="pointer-events-none" />
            </button>
            <button @click="confirmDeleteTemplate(tpl)" class="cursor-pointer p-1 rounded-md text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 transition" title="Delete">
               <AppIcon name="close" :size="12" class="pointer-events-none" />
            </button>
          </div>
        </div>

        <!-- Services -->
        <div class="flex flex-wrap gap-1 mb-3">
          <span
            v-for="svc in tpl.services"
            :key="svc"
            class="text-[9px] bg-[var(--color-surface)] text-[var(--color-muted)] px-1.5 py-0.5 rounded"
          >{{ svc }}</span>
        </div>

        <!-- Actions -->
        <div class="flex items-center justify-between">
          <span class="text-[10px] text-[var(--color-muted)]">{{ tpl.services.length }} services</span>
          <div class="flex gap-2">
            <button @click="viewYaml(tpl)" class="cursor-pointer text-[10px] text-[var(--color-accent)] hover:underline transition">View YAML</button>
            <button @click="launchTemplate(tpl)" class="cursor-pointer flex items-center gap-1.5 text-xs bg-[var(--color-success)]/15 text-[var(--color-success)] px-3 py-1 rounded-lg hover:bg-[var(--color-success)]/25 transition">
              <AppIcon name="play" :size="12" class="pointer-events-none" /> Launch
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Create/Edit Modal -->
    <Teleport to="body">
      <div v-if="showCreateModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" @click.self="closeCreateModal">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-5 w-[520px] max-h-[80vh] overflow-y-auto shadow-2xl">
          <h3 class="text-sm font-semibold text-white mb-3">{{ isEditing ? 'Edit Template' : 'New Template' }}</h3>

          <div class="space-y-3">
            <div>
              <label class="text-[10px] text-[var(--color-muted)] uppercase tracking-wider mb-1 block">Name</label>
              <input v-model="formName" placeholder="e.g. MERN Stack" class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-sm text-white placeholder-[var(--color-muted)] focus:outline-none focus:border-[var(--color-accent)] transition" />
            </div>
            <div>
              <label class="text-[10px] text-[var(--color-muted)] uppercase tracking-wider mb-1 block">Description</label>
              <input v-model="formDescription" placeholder="Short description" class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-sm text-white placeholder-[var(--color-muted)] focus:outline-none focus:border-[var(--color-accent)] transition" />
            </div>
            <div>
              <label class="text-[10px] text-[var(--color-muted)] uppercase tracking-wider mb-1 block">Icon Name (e.g. server, database, code)</label>
              <input v-model="formIcon" placeholder="docker" class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-[var(--color-accent)] transition" />
            </div>
            <div>
              <label class="text-[10px] text-[var(--color-muted)] uppercase tracking-wider mb-1 block">Docker Compose YAML</label>
              <textarea
                v-model="formYaml"
                rows="12"
                placeholder="version: '3.8'&#10;services:&#10;  web:&#10;    image: nginx:alpine&#10;    ports:&#10;      - '8080:80'"
                class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-sm text-white font-mono placeholder-[var(--color-muted)] focus:outline-none focus:border-[var(--color-accent)] transition resize-none"
              ></textarea>
            </div>
          </div>

          <div class="flex gap-2 justify-end mt-4">
            <button @click="closeCreateModal" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Cancel</button>
            <button @click="saveTemplate" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg bg-[var(--color-accent)] text-white hover:opacity-90 transition">
              {{ isEditing ? 'Update' : 'Create' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- YAML Viewer Modal -->
    <Teleport to="body">
      <div v-if="showYamlModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" @click.self="showYamlModal = false">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-5 w-[520px] max-h-[80vh] overflow-y-auto shadow-2xl">
          <h3 class="text-sm font-semibold text-white mb-3">{{ yamlViewTitle }}</h3>
          <pre class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg p-3 text-xs text-gray-300 font-mono whitespace-pre-wrap overflow-auto max-h-96">{{ yamlViewContent }}</pre>
          <div class="flex justify-end mt-3">
            <button @click="showYamlModal = false" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Close</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Delete Confirmation -->
    <Teleport to="body">
      <div v-if="showDeleteModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" @click.self="showDeleteModal = false">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-5 w-80 shadow-2xl">
          <h3 class="text-sm font-semibold text-white mb-2">Delete Template</h3>
          <p class="text-xs text-[var(--color-muted)] mb-4">
            Delete <span class="text-white font-medium">{{ deleteTargetName }}</span>?
          </p>
          <div class="flex gap-2 justify-end">
            <button @click="showDeleteModal = false" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] text-[var(--color-muted)] hover:bg-[var(--color-surface-hover)] transition">Cancel</button>
            <button @click="doDeleteTemplate" class="cursor-pointer px-3 py-1.5 text-xs rounded-lg bg-[var(--color-danger)] text-white hover:opacity-90 transition">Delete</button>
          </div>
        </div>
      </div>
    </Teleport>
    <!-- Launch Progress Modal -->
    <Teleport to="body">
      <div v-if="showLaunchModal" class="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm">
        <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl p-5 w-[600px] shadow-2xl flex flex-col">
          <h3 class="text-sm font-semibold text-white mb-2 flex items-center gap-2">
            <AppIcon name="play" :size="16" class="text-[var(--color-accent)]" />
            Launching {{ launchTargetName }}...
          </h3>
          <pre class="bg-[#0f111a] border border-[var(--color-border)] rounded-lg p-3 text-[11px] text-gray-300 font-mono whitespace-pre-wrap overflow-y-auto h-64 mt-2">{{ launchLogs }}</pre>
          <div class="flex gap-2 justify-end mt-4">
            <button 
              v-if="launching"
              disabled
              class="px-4 py-1.5 text-xs rounded-lg bg-[var(--color-surface)] text-[var(--color-muted)] border border-[var(--color-border)] flex items-center gap-2"
            >
              <div class="animate-spin w-3 h-3 border-2 border-[var(--color-accent)] border-t-transparent rounded-full"></div>
              Starting...
            </button>
            <button 
              v-else
              @click="closeLaunchModal" 
              class="cursor-pointer px-4 py-1.5 text-xs rounded-lg bg-[var(--color-accent)] text-white hover:opacity-90 transition"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import AppIcon from '../components/AppIcon.vue';

interface Template {
  name: string;
  description: string;
  icon: string;
  services: string[];
  yaml: string;
}

const templates = ref<Template[]>([
  {
    name: 'LAMP Stack',
    description: 'Apache, MySQL, PHP – Classic web development',
    icon: 'server',
    services: ['apache', 'mysql', 'phpmyadmin'],
    yaml: `version: "3.8"
services:
  apache:
    image: php:8.2-apache
    ports:
      - "8080:80"
    volumes:
      - ./src:/var/www/html
  mysql:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: root
      MYSQL_DATABASE: app
    ports:
      - "3306:3306"
    volumes:
      - mysql_data:/var/lib/mysql
  phpmyadmin:
    image: phpmyadmin:latest
    ports:
      - "8081:80"
    environment:
      PMA_HOST: mysql
volumes:
  mysql_data:`
  },
  {
    name: 'MERN Stack',
    description: 'MongoDB, Express, React, Node.js',
    icon: 'database',
    services: ['mongo', 'node-api', 'mongo-express'],
    yaml: `version: "3.8"
services:
  mongo:
    image: mongo:7
    ports:
      - "27017:27017"
    volumes:
      - mongo_data:/data/db
  node-api:
    image: node:20-alpine
    working_dir: /app
    volumes:
      - ./api:/app
    ports:
      - "3000:3000"
    command: npm run dev
    depends_on:
      - mongo
  mongo-express:
    image: mongo-express:latest
    ports:
      - "8081:8081"
    environment:
      ME_CONFIG_MONGODB_SERVER: mongo
volumes:
  mongo_data:`
  },
  {
    name: 'Laravel + MySQL',
    description: 'PHP Laravel framework with MySQL & Redis',
    icon: 'layer',
    services: ['nginx', 'php-fpm', 'mysql', 'redis'],
    yaml: `version: "3.8"
services:
  nginx:
    image: nginx:alpine
    ports:
      - "8080:80"
    volumes:
      - ./src:/var/www
      - ./nginx.conf:/etc/nginx/conf.d/default.conf
    depends_on:
      - php
  php:
    image: php:8.2-fpm
    volumes:
      - ./src:/var/www
  mysql:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: root
      MYSQL_DATABASE: laravel
    ports:
      - "3306:3306"
    volumes:
      - mysql_data:/var/lib/mysql
  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
volumes:
  mysql_data:`
  },
  {
    name: 'Python + PostgreSQL',
    description: 'FastAPI / Django with PostgreSQL & Redis',
    icon: 'code',
    services: ['python', 'postgres', 'redis', 'adminer'],
    yaml: `version: "3.8"
services:
  python:
    image: python:3.12-slim
    working_dir: /app
    volumes:
      - ./app:/app
    ports:
      - "8000:8000"
    command: python -m uvicorn main:app --host 0.0.0.0 --reload
    depends_on:
      - postgres
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: app
      POSTGRES_USER: admin
      POSTGRES_PASSWORD: admin
    ports:
      - "5432:5432"
    volumes:
      - pg_data:/var/lib/postgresql/data
  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
  adminer:
    image: adminer:latest
    ports:
      - "8080:8080"
volumes:
  pg_data:`
  }
]);

// Create/Edit form
const showCreateModal = ref(false);
const isEditing = ref(false);
const editingIndex = ref(-1);
const formName = ref('');
const formDescription = ref('');
const formIcon = ref('docker');
const formYaml = ref('');

// YAML viewer
const showYamlModal = ref(false);
const yamlViewTitle = ref('');
const yamlViewContent = ref('');

// Delete
const showDeleteModal = ref(false);
const deleteTargetName = ref('');
const deleteTargetIndex = ref(-1);

function parseServices(yaml: string): string[] {
  const services: string[] = [];
  const lines = yaml.split('\n');
  let inServices = false;
  let serviceIndent = -1;
  for (const line of lines) {
    if (line.trim() === 'services:') {
      inServices = true;
      serviceIndent = line.indexOf('s');
      continue;
    }
    if (inServices) {
      const trimmed = line.trimEnd();
      if (trimmed.length === 0) continue;
      const indent = trimmed.length - trimmed.trimStart().length;
      if (indent <= serviceIndent && trimmed.trim() !== '') { inServices = false; continue; }
      if (indent === serviceIndent + 2 && trimmed.endsWith(':')) {
        services.push(trimmed.trim().replace(':', ''));
      }
    }
  }
  return services;
}

function editTemplate(tpl: Template) {
  const idx = templates.value.indexOf(tpl);
  isEditing.value = true;
  editingIndex.value = idx;
  formName.value = tpl.name;
  formDescription.value = tpl.description;
  formIcon.value = tpl.icon;
  formYaml.value = tpl.yaml;
  showCreateModal.value = true;
}

function closeCreateModal() {
  showCreateModal.value = false;
  isEditing.value = false;
  editingIndex.value = -1;
  formName.value = '';
  formDescription.value = '';
  formIcon.value = 'docker';
  formYaml.value = '';
}

function saveTemplate() {
  if (!formName.value.trim() || !formYaml.value.trim()) return;
  const tpl: Template = {
    name: formName.value.trim(),
    description: formDescription.value.trim(),
    icon: formIcon.value || 'docker',
    services: parseServices(formYaml.value),
    yaml: formYaml.value,
  };
  if (isEditing.value && editingIndex.value >= 0) {
    templates.value[editingIndex.value] = tpl;
  } else {
    templates.value.push(tpl);
  }
  closeCreateModal();
}

function viewYaml(tpl: Template) {
  yamlViewTitle.value = tpl.name;
  yamlViewContent.value = tpl.yaml;
  showYamlModal.value = true;
}

function confirmDeleteTemplate(tpl: Template) {
  deleteTargetName.value = tpl.name;
  deleteTargetIndex.value = templates.value.indexOf(tpl);
  showDeleteModal.value = true;
}

function doDeleteTemplate() {
  if (deleteTargetIndex.value >= 0) {
    templates.value.splice(deleteTargetIndex.value, 1);
  }
  showDeleteModal.value = false;
}

// Launch
const showLaunchModal = ref(false);
const launchTargetName = ref('');
const launchLogs = ref('');
const launching = ref(false);

async function launchTemplate(tpl: Template) {
  showLaunchModal.value = true;
  launchTargetName.value = tpl.name;
  launchLogs.value = 'Starting docker compose up...\n';
  launching.value = true;
  
  let unlisten: (() => void) | undefined;
  try {
    unlisten = await listen<string>('compose-progress', (event) => {
      launchLogs.value += event.payload + '\n';
    });
    const result = await invoke<string>('launch_compose', { yaml: tpl.yaml, projectName: tpl.name.toLowerCase().replace(/\s+/g, '-') });
    launchLogs.value += `\nSuccess: ${result}`;
  } catch (e) {
    launchLogs.value += `\nError: ${e}`;
    console.error('Launch failed', e);
  } finally {
    launching.value = false;
    if (unlisten) unlisten();
  }
}

function closeLaunchModal() {
  showLaunchModal.value = false;
  launchLogs.value = '';
}
</script>
