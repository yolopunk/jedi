<template>
    <div class="hosts-panel">
        <!-- Toolbar -->
        <div class="hosts-toolbar">
            <div class="search-field">
                <v-icon :icon="mdiMagnify" size="16" class="search-icon" />
                <input
                    v-model="searchModel"
                    type="text"
                    class="search-input"
                    :placeholder="searchPlaceholder"
                    @input="handleSearch"
                />
            </div>
            <button class="add-btn" @click="emit('add-host', currentGroup.name)">
                <v-icon :icon="mdiPlus" size="16" />
                <span>{{ addHostText }}</span>
            </button>
        </div>

        <!-- Table -->
        <div class="hosts-scroll" ref="tableWrapperRef">
            <table v-if="filteredItems.length" class="hosts-table">
                <colgroup>
                    <col class="col-ip" />
                    <col class="col-domain" />
                    <col class="col-status" />
                    <col class="col-actions" />
                </colgroup>
                <thead>
                    <tr>
                        <th>{{ $t("hosts.console.ipAddress") }}</th>
                        <th>{{ $t("hosts.console.domain") }}</th>
                        <th>{{ $t("hosts.console.status") }}</th>
                        <th class="th-actions">{{ $t("hosts.console.actions") }}</th>
                    </tr>
                </thead>
                <tbody>
                    <tr
                        v-for="item in displayItems"
                        :key="item.id"
                        class="data-row"
                        :class="{ disabled: !item.enabled }"
                    >
                        <td><span class="mono">{{ item.ip }}</span></td>
                        <td>
                            <button
                                class="domain-link"
                                @click="handleOpenDomain(item.domain)"
                                :title="item.domain"
                            >
                                <span class="mono">{{ item.domain }}</span>
                            </button>
                        </td>
                        <td>
                            <button
                                type="button"
                                role="switch"
                                class="row-switch"
                                :class="{ on: item.enabled }"
                                :aria-checked="item.enabled"
                                :aria-label="item.enabled ? $t('hosts.table.active') : $t('hosts.console.status')"
                                @click="toggleItemEnabled(item)"
                            >
                                <span class="knob"></span>
                            </button>
                        </td>
                        <td class="td-actions">
                            <button
                                class="icon-action"
                                @click="emit('edit-host', item.originalMap)"
                                :title="$t('common.edit')"
                            >
                                <v-icon :icon="mdiPencilOutline" size="16" />
                            </button>
                            <button
                                class="icon-action danger"
                                @click="emit('delete-host', item.originalMap)"
                                :title="$t('common.delete')"
                            >
                                <v-icon :icon="mdiTrashCanOutline" size="16" />
                            </button>
                        </td>
                    </tr>
                </tbody>
            </table>

            <!-- Loading -->
            <div v-if="loading" class="hosts-state">
                <v-progress-circular indeterminate size="20" width="2" color="primary" />
                <span>{{ $t("common.loading") }}</span>
            </div>

            <!-- Empty -->
            <div v-else-if="!filteredItems.length" class="hosts-state empty">
                <v-icon :icon="mdiServerNetworkOff" size="32" />
                <span>{{ $t("hosts.empty.noEntries") }}</span>
            </div>

            <!-- Load more -->
            <button
                v-if="hasMore && !loading"
                class="load-more"
                @click="loadMore"
            >
                {{ $t("hosts.table.loadMore") }}
            </button>
        </div>

        <!-- Status line -->
        <div class="hosts-statusbar">
            <span class="group-label">{{ currentGroup.name }}</span>
            <span class="counts">
                {{ filteredItems.length }} {{ $t("hosts.table.entries") }}
                <span class="sep">·</span>
                {{ activeCount }} {{ $t("hosts.table.active") }}
            </span>
        </div>
    </div>
</template>

<script setup lang="ts">
import {
  mdiMagnify,
  mdiPencilOutline,
  mdiPlus,
  mdiServerNetworkOff,
  mdiTrashCanOutline,
} from '@mdi/js'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Group, HostEntry } from '@/types/hosts'
import { getHostsAsItems, openDomainLink } from '@/utils/hostsUtils'

const { t } = useI18n()

const props = defineProps<{
  currentGroup: Group
  search?: string
  loading?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:search', value: string): void
  (e: 'update-status', host: HostEntry, enabled: boolean): void
  (e: 'edit-host', host: HostEntry): void
  (e: 'delete-host', host: HostEntry): void
  (e: 'add-host', name: string): void
  (e: 'open-domain', domain: string, message: string): void
}>()

const tableWrapperRef = ref<HTMLElement | null>(null)
const itemsPerPage = 15
const currentDisplayCount = ref(itemsPerPage)

const searchModel = computed({
  get: () => props.search || '',
  set: value => emit('update:search', value),
})

const filteredItems = computed(() => {
  if (props.loading && !props.currentGroup?.hosts) {
    return []
  }
  return getHostsAsItems(props.currentGroup.hosts)
})

const activeCount = computed(() => {
  return filteredItems.value.filter(item => item.enabled).length
})

const displayItems = computed(() => {
  return filteredItems.value.slice(0, currentDisplayCount.value)
})

const hasMore = computed(() => {
  return currentDisplayCount.value < filteredItems.value.length
})

// Use t() to avoid TS6133
const searchPlaceholder = computed(() => t('common.search'))
const addHostText = computed(() => t('hosts.table.addHost'))

watch(filteredItems, () => {
  currentDisplayCount.value = itemsPerPage
})

function loadMore() {
  if (hasMore.value) {
    currentDisplayCount.value += itemsPerPage
  }
}

function handleSearch() {
  currentDisplayCount.value = itemsPerPage
}

function toggleItemEnabled(item: {
  domain: string
  ip: string
  enabled: boolean
  originalMap: HostEntry
}) {
  item.enabled = !item.enabled
  emit('update-status', item.originalMap, item.enabled)
}

function handleOpenDomain(domain: string) {
  openDomainLink(domain)
    .then(message => {
      emit('open-domain', domain, message)
    })
    .catch(error => {
      console.error('打开域名失败:', error)
      emit('open-domain', domain, `打开域名失败: ${domain}`)
    })
}

let scrollHandler: (() => void) | null = null

function setupScrollListener() {
  if (!tableWrapperRef.value) return

  const wrapper = tableWrapperRef.value
  scrollHandler = () => {
    const { scrollTop, scrollHeight, clientHeight } = wrapper
    if (scrollTop + clientHeight >= scrollHeight - 50) {
      loadMore()
    }
  }
  wrapper.addEventListener('scroll', scrollHandler)
}

function removeScrollListener() {
  if (!tableWrapperRef.value || !scrollHandler) return
  tableWrapperRef.value.removeEventListener('scroll', scrollHandler)
  scrollHandler = null
}

onMounted(() => {
  setTimeout(setupScrollListener, 100)
})

onUnmounted(() => {
  removeScrollListener()
})
</script>

<style scoped>
/* Clean IDE data table — quiet, dense, scannable */
.hosts-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
}

/* Toolbar */
.hosts-toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
}

.search-field {
    position: relative;
    display: flex;
    align-items: center;
    flex: 1;
    max-width: 360px;
}

.search-icon {
    position: absolute;
    left: 9px;
    color: var(--text-subtle);
    pointer-events: none;
}

.search-input {
    width: 100%;
    height: 30px;
    padding: 0 10px 0 30px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-size: 13px;
    font-family: inherit;
    outline: none;
    transition: border-color 0.12s ease, box-shadow 0.12s ease;
}

.search-input::placeholder { color: var(--text-subtle); }

.search-input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgb(var(--accent-rgb) / 0.18);
}

.add-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 12px;
    background: var(--accent);
    color: var(--on-accent);
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.12s ease;
}

.add-btn:hover { background: var(--accent-hover); }

/* Scroll region */
.hosts-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
}

/* Table */
.hosts-table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
}

.col-status { width: 88px; }
.col-actions { width: 84px; }

.hosts-table thead th {
    position: sticky;
    top: 0;
    z-index: 1;
    background: var(--bg);
    text-align: left;
    padding: 8px 16px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-subtle);
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
}

.th-actions { text-align: right; }

.data-row td {
    padding: 0 16px;
    height: 40px;
    border-bottom: 1px solid var(--border);
    font-size: 13px;
    color: var(--text);
    vertical-align: middle;
}

.data-row:hover td { background: rgb(var(--ink-rgb) / 0.03); }

.mono {
    font-family: var(--jedi-font-mono);
    font-size: 12.5px;
}

.data-row td:first-child .mono { color: var(--text-muted); }

.domain-link {
    display: inline-block;
    max-width: 100%;
    background: none;
    border: none;
    padding: 0;
    color: var(--text);
    cursor: pointer;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.domain-link:hover { color: var(--accent); text-decoration: underline; }

/* Disabled row reads quietly muted */
.data-row.disabled .mono,
.data-row.disabled .domain-link { color: var(--text-subtle); }

/* Signature: enable/disable switch */
.row-switch {
    position: relative;
    width: 34px;
    height: 18px;
    border-radius: 999px;
    border: none;
    background: rgb(var(--ink-rgb) / 0.22);
    cursor: pointer;
    padding: 0;
    transition: background 0.16s ease;
    flex-shrink: 0;
}

.row-switch .knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 2px rgb(var(--ink-rgb) / 0.3);
    transition: transform 0.16s ease;
}

.row-switch.on { background: var(--accent); }
.row-switch.on .knob { transform: translateX(16px); }

.row-switch:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
}

/* Action icons — ghost buttons */
.td-actions {
    text-align: right;
    white-space: nowrap;
}

.icon-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: none;
    border-radius: 5px;
    background: none;
    color: var(--text-subtle);
    cursor: pointer;
    transition: background 0.12s ease, color 0.12s ease;
}

.icon-action:hover {
    background: rgb(var(--ink-rgb) / 0.06);
    color: var(--text);
}

.icon-action.danger:hover {
    background: rgb(var(--danger-rgb) / 0.1);
    color: var(--danger);
}

/* States */
.hosts-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 64px 16px;
    color: var(--text-subtle);
    font-size: 13px;
}

.hosts-state.empty .v-icon { color: var(--text-subtle); opacity: 0.6; }

.load-more {
    display: block;
    width: 100%;
    padding: 10px;
    background: none;
    border: none;
    border-top: 1px solid var(--border);
    color: var(--accent);
    font-size: 13px;
    cursor: pointer;
}

.load-more:hover { background: rgb(var(--ink-rgb) / 0.03); }

/* Status line */
.hosts-statusbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    border-top: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-subtle);
}

.hosts-statusbar .group-label {
    font-weight: 500;
    color: var(--text-muted);
}

.hosts-statusbar .sep { margin: 0 4px; opacity: 0.6; }
</style>
