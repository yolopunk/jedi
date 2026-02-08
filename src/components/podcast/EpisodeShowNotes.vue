<template>
  <div class="episode-show-notes">
    <div class="d-flex align-center justify-space-between mb-2">
      <h3 class="text-subtitle-1 font-weight-bold">Show Notes</h3>
      <v-btn
        v-if="collapsible"
        :icon="expanded ? mdiChevronUp : mdiChevronDown"
        variant="text"
        density="compact"
        @click="expanded = !expanded"
      ></v-btn>
    </div>
    
    <v-expand-transition>
      <div v-show="expanded || !collapsible">
        <div 
          class="show-notes-content text-body-2" 
          v-html="content"
          @click="handleClick"
        ></div>
      </div>
    </v-expand-transition>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { mdiChevronDown, mdiChevronUp } from '@mdi/js';
import { open } from '@tauri-apps/plugin-shell';

const props = defineProps({
  content: {
    type: String,
    required: true
  },
  collapsible: {
    type: Boolean,
    default: false
  },
  defaultExpanded: {
    type: Boolean,
    default: true
  }
});

const expanded = ref(props.defaultExpanded);

function handleClick(event: MouseEvent) {
  const target = event.target as HTMLElement;
  const link = target.closest('a');
  if (link) {
    event.preventDefault();
    const href = link.getAttribute('href');
    if (href) {
      open(href).catch(err => {
        console.error('Failed to open link:', err);
        window.open(href, '_blank');
      });
    }
  }
}
</script>

<style>
.show-notes-content {
  line-height: 1.6;
}
.show-notes-content img {
  max-width: 100%;
  height: auto;
  border-radius: 8px;
  margin: 8px 0;
  display: block;
}
.show-notes-content a {
  color: rgb(var(--v-theme-primary));
  text-decoration: none;
}
.show-notes-content a:hover {
  text-decoration: underline;
}
.show-notes-content p {
  margin-bottom: 0.75rem;
}
.show-notes-content ul, .show-notes-content ol {
  padding-left: 1.5rem;
  margin-bottom: 0.75rem;
}
.show-notes-content blockquote {
  border-left: 4px solid rgba(var(--v-border-color), var(--v-border-opacity));
  padding-left: 1rem;
  margin: 1rem 0;
  font-style: italic;
}
</style>
