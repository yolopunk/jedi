<template>
  <div class="episode-show-notes">
    <div v-if="collapsible" class="d-flex align-center justify-space-between mb-4">
      <h3 class="text-h6 font-weight-bold">Show Notes</h3>
      <v-btn
        :icon="expanded ? mdiChevronUp : mdiChevronDown"
        variant="text"
        density="compact"
        @click="expanded = !expanded"
      ></v-btn>
    </div>
    
    <v-expand-transition>
      <div v-show="expanded || !collapsible">
        <div 
          class="show-notes-content text-body-1" 
          v-html="formattedContent"
          @click="handleClick"
        ></div>
      </div>
    </v-expand-transition>
  </div>
</template>

<script setup lang="ts">
import { mdiChevronDown, mdiChevronUp } from '@mdi/js'
import { open } from '@tauri-apps/plugin-shell'
import { computed, ref } from 'vue'

const props = defineProps({
  content: {
    type: String,
    required: true,
  },
  collapsible: {
    type: Boolean,
    default: false,
  },
  defaultExpanded: {
    type: Boolean,
    default: true,
  },
})

const emit = defineEmits(['seek'])

const expanded = ref(props.defaultExpanded)

const formattedContent = computed(() => {
  if (!props.content) return ''

  let content = props.content

  // If content seems to be plain text (no html tags like <p>, <div, <br), convert newlines to <br>
  if (!content.match(/<[a-z][\s\S]*>/i)) {
    content = content.replace(/\n/g, '<br>')
  }

  // Highlight timestamps (e.g. 05:00, 12:30, 1:05:00)
  // Wrap them in a span that we can style and click
  content = content.replace(
    /\b((?:\d{1,2}:)?\d{1,2}:\d{2})\b/g,
    '<span class="timestamp-link text-primary font-weight-bold cursor-pointer" data-timestamp="$1">$1</span>'
  )

  return content
})

function handleClick(event: MouseEvent) {
  const target = event.target as HTMLElement

  // Handle Timestamp Clicks
  if (target.classList.contains('timestamp-link')) {
    const timeStr = target.dataset.timestamp
    if (timeStr) {
      event.preventDefault()
      event.stopPropagation()

      // Convert to seconds
      const parts = timeStr.split(':').reverse()
      let seconds = 0
      if (parts[0]) seconds += parseInt(parts[0], 10)
      if (parts[1]) seconds += parseInt(parts[1], 10) * 60
      if (parts[2]) seconds += parseInt(parts[2], 10) * 3600

      emit('seek', seconds)
      return
    }
  }

  // Handle Links
  const link = target.closest('a')
  if (link) {
    event.preventDefault()
    const href = link.getAttribute('href')
    if (href) {
      open(href).catch(err => {
        console.error('Failed to open link:', err)
        window.open(href, '_blank')
      })
    }
  }
}
</script>

<style>
.show-notes-content {
  line-height: 1.8;
  color: rgb(var(--text-rgb) / 0.9);
}
.show-notes-content img {
  max-width: 100%;
  height: auto;
  border-radius: 12px;
  margin: 16px 0;
  display: block;
  box-shadow: 0 4px 6px rgba(0,0,0,0.1);
}
.show-notes-content a {
  color: rgb(var(--v-theme-primary));
  text-decoration: none;
  border-bottom: 1px dotted rgba(var(--v-theme-primary), 0.5);
}
.show-notes-content a:hover {
  text-decoration: none;
  border-bottom-style: solid;
}
.show-notes-content p {
  margin-bottom: 1rem;
}
.show-notes-content ul, .show-notes-content ol {
  padding-left: 1.5rem;
  margin-bottom: 1rem;
}
.show-notes-content blockquote {
  border-left: 4px solid rgba(var(--v-theme-primary), 0.5);
  background:rgb(var(--ink-rgb) / 0.05);
  padding: 1rem;
  margin: 1.5rem 0;
  border-radius: 0 8px 8px 0;
  font-style: italic;
}
.show-notes-content h1, 
.show-notes-content h2, 
.show-notes-content h3, 
.show-notes-content h4 {
    margin-top: 2rem;
    margin-bottom: 1rem;
    font-weight: 700;
    line-height: 1.3;
}

/* Timestamp styling */
.timestamp-link {
    display: inline-block;
    padding: 0 4px;
    margin: 0 2px;
    border-radius: 4px;
    transition: all 0.2s;
    background: rgba(var(--v-theme-primary), 0.1);
}
.timestamp-link:hover {
    background: rgba(var(--v-theme-primary), 0.2);
    transform: translateY(-1px);
}
.cursor-pointer {
    cursor: pointer;
}
</style>
