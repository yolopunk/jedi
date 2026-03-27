<template>
    <div class="hosts-console-container">
        <!-- CRT Effects -->
        <div class="scanlines"></div>
        <div class="crt-vignette"></div>

        <!-- Toolbar -->
        <div class="console-toolbar">
            <div class="toolbar-section">
                <div class="input-wrapper">
                    <span class="input-prompt">>></span>
                    <input
                        v-model="searchModel"
                        type="text"
                        class="terminal-input"
                        :placeholder="searchPlaceholder + '...'"
                        @input="handleSearch"
                    />
                    <span class="input-cursor"></span>
                </div>
            </div>
            <div class="toolbar-section">
                <button
                    class="console-btn btn-primary"
                    @click="emit('add-host', currentGroup.name)"
                >
                    <span class="btn-icon">+</span>
                    <span class="btn-text">{{ addHostText }}</span>
                </button>
            </div>
        </div>

        <!-- Grid Background -->
        <div class="grid-background"></div>

        <!-- Table Container -->
        <div class="table-wrapper" ref="tableWrapperRef">
            <!-- Table Header -->
            <div class="table-header-row">
                <div class="table-cell cell-ip">
                    <span class="cell-label">{{
                        $t("hosts.console.ipAddress")
                    }}</span>
                    <span class="cell-divider">|</span>
                </div>
                <div class="table-cell cell-domain">
                    <span class="cell-label">{{
                        $t("hosts.console.domain")
                    }}</span>
                    <span class="cell-divider">|</span>
                </div>
                <div class="table-cell cell-status">
                    <span class="cell-label">{{
                        $t("hosts.console.status")
                    }}</span>
                    <span class="cell-divider">|</span>
                </div>
                <div class="table-cell cell-actions">
                    <span class="cell-label">{{
                        $t("hosts.console.actions")
                    }}</span>
                </div>
            </div>

            <!-- Table Body -->
            <div class="table-body">
                <div
                    v-for="(item, index) in displayItems"
                    :key="item.id"
                    class="table-row"
                    :class="{ 'row-highlight': index % 2 === 0 }"
                    data-row-index="index"
                >
                    <div class="table-cell cell-ip">
                        <span class="row-index"
                            >[{{ String(index + 1).padStart(3, "0") }}]</span
                        >
                        <span class="ip-address">{{ item.ip }}</span>
                    </div>
                    <div class="table-cell cell-domain">
                        <span class="domain-name">{{ item.domain }}</span>
                        <button
                            class="icon-btn"
                            @click="handleOpenDomain(item.domain)"
                            title="Open Domain"
                        >
                            <span class="icon-globe">◈</span>
                        </button>
                    </div>
                    <div class="table-cell cell-status">
                        <div
                            class="status-toggle"
                            :class="{ 'status-active': item.enabled }"
                            @click="toggleItemEnabled(item)"
                        >
                            <div class="status-core">
                                <div
                                    class="status-inner"
                                    :class="{ 'glow-active': item.enabled }"
                                ></div>
                            </div>
                            <span class="status-text">{{
                                item.enabled ? "ACTIVE" : "INACTIVE"
                            }}</span>
                        </div>
                    </div>
                    <div class="table-cell cell-actions">
                        <button
                            class="icon-btn btn-edit"
                            @click="emit('edit-host', item)"
                            title="Edit"
                        >
                            <span class="icon-edit">✎</span>
                        </button>
                        <button
                            class="icon-btn btn-delete"
                            @click="emit('delete-host', item.id)"
                            title="Delete"
                        >
                            <span class="icon-delete">✕</span>
                        </button>
                    </div>
                </div>

                <!-- Loading Indicator -->
                <div v-if="loading" class="loading-row">
                    <div class="loading-content">
                        <div class="loading-spinner">
                            <div class="spinner-segment"></div>
                            <div class="spinner-segment"></div>
                            <div class="spinner-segment"></div>
                            <div class="spinner-segment"></div>
                        </div>
                        <span class="loading-text"
                            >INITIALIZING_HOSTS_MATRIX...</span
                        >
                    </div>
                </div>

                <!-- Load More -->
                <div
                    v-if="hasMore && !loading"
                    class="load-more-row"
                    @click="loadMore"
                >
                    <div class="load-more-content">
                        <span class="load-more-text"
                            >[ LOAD_MORE_ENTRIES ]</span
                        >
                        <div class="load-more-line"></div>
                    </div>
                </div>

                <!-- No More Data -->
                <div
                    v-else-if="!hasMore && filteredItems.length > 0"
                    class="end-row"
                >
                    <div class="end-content">
                        <span class="end-line"
                            >══════════════════════════════════════════════════════════════</span
                        >
                        <span class="end-text">END_OF_TRANSMISSION</span>
                        <span class="end-line"
                            >══════════════════════════════════════════════════════════════</span
                        >
                    </div>
                </div>

                <!-- Empty State -->
                <div
                    v-if="filteredItems.length === 0 && !loading"
                    class="empty-row"
                >
                    <div class="empty-content">
                        <div class="empty-icon">◇</div>
                        <span class="empty-text">NO_HOST_ENTRIES_FOUND</span>
                        <span class="empty-hint"
                            >Use the + button to add a new host entry</span
                        >
                    </div>
                </div>
            </div>
        </div>

        <!-- Footer Status Bar -->
        <div class="console-footer">
            <div class="footer-section">
                <span class="footer-label">GROUP:</span>
                <span class="footer-value">{{
                    currentGroup.name || "DEFAULT"
                }}</span>
            </div>
            <div class="footer-section">
                <span class="title-prefix">[</span>
                <span class="title-text">{{ $t("hosts.console.title") }}</span>
                <span class="title-suffix">]</span>
            </div>
            <div class="footer-section">
                <span class="footer-label">DISPLAYING:</span>
                <span class="footer-value"
                    >{{ displayItems.length }} /
                    {{ filteredItems.length }}</span
                >
            </div>
            <div class="footer-section">
                <span class="metric-item">
                    <span class="metric-label"
                        >{{ $t("hosts.table.entries") }}:</span
                    >
                    <span class="metric-value">{{ filteredItems.length }}</span>
                </span>
                <span class="metric-item">
                    <span class="metric-label"
                        >{{ $t("hosts.table.active") }}:</span
                    >
                    <span class="metric-value metric-active">{{
                        activeCount
                    }}</span>
                </span>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { useI18n } from "vue-i18n";
import { getHostsAsItems, openDomainLink } from "@/utils/hostsUtils";
import { Group } from "@/types/hosts";

const { t } = useI18n();

const props = defineProps<{
    currentGroup: Group;
    search?: string;
    loading?: boolean;
}>();

const emit = defineEmits<{
    (e: "update:search", value: string): void;
    (e: "update-status", host: any): void;
    (e: "edit-host", host: any): void;
    (e: "delete-host", host: any): void;
    (e: "add-host", name: string): void;
    (e: "open-domain", domain: string, message: string): void;
}>();

const tableWrapperRef = ref<HTMLElement | null>(null);
const itemsPerPage = 15;
const currentDisplayCount = ref(itemsPerPage);

const searchModel = computed({
    get: () => props.search || "",
    set: (value) => emit("update:search", value),
});

const filteredItems = computed(() => {
    if (props.loading && (!props.currentGroup || !props.currentGroup.hosts)) {
        return [];
    }
    return getHostsAsItems(props.currentGroup.hosts);
});

const activeCount = computed(() => {
    return filteredItems.value.filter((item) => item.enabled).length;
});

const displayItems = computed(() => {
    return filteredItems.value.slice(0, currentDisplayCount.value);
});

const hasMore = computed(() => {
    return currentDisplayCount.value < filteredItems.value.length;
});

// Use t() to avoid TS6133
const searchPlaceholder = computed(() => t("common.search"));
const addHostText = computed(() => t("hosts.table.addHost"));

watch(filteredItems, () => {
    currentDisplayCount.value = itemsPerPage;
});

function loadMore() {
    if (hasMore.value) {
        currentDisplayCount.value += itemsPerPage;
    }
}

function handleSearch() {
    currentDisplayCount.value = itemsPerPage;
}

function toggleItemEnabled(item: any) {
    item.enabled = !item.enabled;
    emit("update-status", item);
}

function handleOpenDomain(domain: string) {
    openDomainLink(domain)
        .then((message) => {
            emit("open-domain", domain, message);
        })
        .catch((error) => {
            console.error("打开域名失败:", error);
            emit("open-domain", domain, `打开域名失败: ${domain}`);
        });
}

let scrollHandler: (() => void) | null = null;

function setupScrollListener() {
    if (!tableWrapperRef.value) return;

    const wrapper = tableWrapperRef.value;
    scrollHandler = () => {
        const { scrollTop, scrollHeight, clientHeight } = wrapper;
        if (scrollTop + clientHeight >= scrollHeight - 50) {
            loadMore();
        }
    };
    wrapper.addEventListener("scroll", scrollHandler);
}

function removeScrollListener() {
    if (!tableWrapperRef.value || !scrollHandler) return;
    tableWrapperRef.value.removeEventListener("scroll", scrollHandler);
    scrollHandler = null;
}

onMounted(() => {
    setTimeout(setupScrollListener, 100);
});

onUnmounted(() => {
    removeScrollListener();
});
</script>

<style scoped>
.hosts-console-container {
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: linear-gradient(180deg, #0a0a0f 0%, #0d0d14 50%, #0a0a0f 100%);
    border: 1px solid #1a1a2e;
    border-radius: 8px;
    overflow: hidden;
    font-family: "SF Mono", "Monaco", "Inconsolata", "Roboto Mono", monospace;
}

/* CRT Effects */
.scanlines {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    pointer-events: none;
    z-index: 100;
    background: repeating-linear-gradient(
        0deg,
        rgba(0, 0, 0, 0.12),
        rgba(0, 0, 0, 0.12) 1px,
        transparent 1px,
        transparent 2px
    );
}

.crt-vignette {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    pointer-events: none;
    z-index: 99;
    background: radial-gradient(
        ellipse at center,
        transparent 0%,
        rgba(0, 0, 0, 0.35) 100%
    );
}

/* Grid Background */
.grid-background {
    position: absolute;
    top: 80px;
    left: 0;
    right: 0;
    bottom: 40px;
    pointer-events: none;
    z-index: 1;
    background-image:
        linear-gradient(rgba(0, 255, 255, 0.03) 1px, transparent 1px),
        linear-gradient(90deg, rgba(0, 255, 255, 0.03) 1px, transparent 1px);
    background-size: 40px 40px;
}

.title-prefix,
.title-suffix {
    color: #00ffff;
    font-size: 12px;
    text-shadow: 0 0 10px #00ffff88;
}

.title-text {
    color: #00ff88;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 2px;
    text-shadow: 0 0 10px #00ff8888;
}

.console-metrics {
    display: flex;
    gap: 16px;
}

.metric-item {
    display: flex;
    align-items: center;
    gap: 4px;
}

.metric-label {
    color: #666;
    font-size: 10px;
    letter-spacing: 1px;
}

.metric-value {
    color: #00ffff;
    font-size: 12px;
    font-weight: 700;
    text-shadow: 0 0 8px #00ffff66;
}

.metric-active {
    color: #00ff88;
    text-shadow: 0 0 8px #00ff8866;
}

/* Toolbar */
.console-toolbar {
    position: relative;
    z-index: 5;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: rgba(10, 10, 20, 0.8);
    border-bottom: 1px solid #1a1a3a;
}

.toolbar-section {
    display: flex;
    align-items: center;
    gap: 12px;
}

.input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
    background: #050508;
    border: 1px solid #1a1a3a;
    border-radius: 4px;
    padding: 6px 12px;
    min-width: 280px;
}

.input-prompt {
    color: #00ff88;
    font-size: 12px;
    margin-right: 8px;
    text-shadow: 0 0 8px #00ff8866;
}

.terminal-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: #00ffff;
    font-family: inherit;
    font-size: 12px;
    width: 180px;
}

.terminal-input::placeholder {
    color: #333;
}

.input-cursor {
    width: 8px;
    height: 14px;
    background: #00ffff;
    margin-left: 4px;
    animation: cursor-blink 1s step-end infinite;
}

@keyframes cursor-blink {
    0%,
    100% {
        opacity: 1;
    }
    50% {
        opacity: 0;
    }
}

.console-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: 1px solid #00ff88;
    color: #00ff88;
    padding: 8px 16px;
    border-radius: 4px;
    font-family: inherit;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 1px;
    cursor: pointer;
    transition: all 0.2s ease;
}

.console-btn:hover {
    background: #00ff8811;
    box-shadow:
        0 0 15px #00ff8844,
        inset 0 0 15px #00ff8811;
}

.btn-icon {
    font-size: 14px;
}

/* Table Wrapper */
.table-wrapper {
    position: relative;
    z-index: 5;
    flex: 1;
    overflow-y: auto;
    min-height: 0;
}

.table-wrapper::-webkit-scrollbar {
    width: 8px;
}

.table-wrapper::-webkit-scrollbar-track {
    background: #0a0a0f;
}

.table-wrapper::-webkit-scrollbar-thumb {
    background: #00ffff33;
    border-radius: 4px;
}

.table-wrapper::-webkit-scrollbar-thumb:hover {
    background: #00ffff55;
}

/* Table Header */
.table-header-row {
    display: flex;
    align-items: center;
    background: linear-gradient(180deg, #0f0f1a 0%, #0a0a12 100%);
    border-bottom: 2px solid #00ffff44;
    padding: 12px 16px;
    position: sticky;
    top: 0;
    z-index: 10;
}

.table-cell {
    display: flex;
    align-items: center;
    gap: 8px;
}

.cell-ip {
    width: 200px;
    flex-shrink: 0;
}

.cell-domain {
    flex: 1;
    min-width: 0;
}

.cell-status {
    width: 140px;
    flex-shrink: 0;
    justify-content: center;
}

.cell-actions {
    width: 100px;
    flex-shrink: 0;
    justify-content: center;
    gap: 4px;
}

.cell-label {
    color: #00ffff;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 2px;
    text-shadow: 0 0 8px #00ffff66;
}

.cell-divider {
    color: #00ffff33;
    font-size: 12px;
}

/* Table Body */
.table-body {
    position: relative;
}

.table-row {
    display: flex;
    align-items: center;
    padding: 10px 16px;
    border-bottom: 1px solid #1a1a2e;
    transition: all 0.2s ease;
    position: relative;
}

.table-row:hover {
    background: #00ffff08;
}

.table-row:hover::before {
    content: "";
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background: #00ffff;
    box-shadow: 0 0 15px #00ffff;
}

.row-highlight {
    background: #00000033;
}

.row-index {
    color: #444;
    font-size: 10px;
    margin-right: 8px;
    min-width: 40px;
}

.ip-address {
    color: #ff00ff;
    font-size: 12px;
    font-weight: 600;
    text-shadow: 0 0 8px #ff00ff44;
}

.domain-name {
    color: #00ff88;
    font-size: 12px;
    text-shadow: 0 0 8px #00ff8844;
}

/* Status Toggle */
.status-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    transition: all 0.2s ease;
}

.status-toggle:hover {
    background: #ffffff08;
}

.status-core {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #1a1a2e;
    border: 2px solid #333;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.3s ease;
}

.status-inner {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #555;
    transition: all 0.3s ease;
}

.status-active .status-core {
    border-color: #00ff88;
    box-shadow: 0 0 10px #00ff8844;
}

.status-active .status-inner {
    background: #00ff88;
}

.status-active .glow-active {
    animation: status-glow 1.5s ease-in-out infinite;
}

@keyframes status-glow {
    0%,
    100% {
        box-shadow:
            0 0 8px #00ff88,
            0 0 16px #00ff8866;
    }
    50% {
        box-shadow:
            0 0 12px #00ff88,
            0 0 24px #00ff88aa;
    }
}

.status-text {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 1px;
    color: #666;
    transition: all 0.3s ease;
}

.status-active .status-text {
    color: #00ff88;
    text-shadow: 0 0 8px #00ff8866;
}

/* Icon Buttons */
.icon-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid #333;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s ease;
    font-size: 12px;
}

.icon-btn:hover {
    border-color: #00ffff;
    box-shadow: 0 0 10px #00ffff44;
}

.icon-globe {
    color: #00ffff;
}

.icon-edit {
    color: #ffaa00;
}

.btn-edit:hover {
    border-color: #ffaa00;
    box-shadow: 0 0 10px #ffaa0044;
}

.icon-delete {
    color: #ff4444;
}

.btn-delete:hover {
    border-color: #ff4444;
    box-shadow: 0 0 10px #ff444444;
}

/* Loading Row */
.loading-row {
    padding: 32px 16px;
    display: flex;
    justify-content: center;
}

.loading-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
}

.loading-spinner {
    width: 40px;
    height: 40px;
    position: relative;
}

.spinner-segment {
    position: absolute;
    width: 100%;
    height: 100%;
    border: 2px solid transparent;
    border-top-color: #00ffff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

.spinner-segment:nth-child(2) {
    border-top-color: #00ff88;
    animation-delay: 0.15s;
    width: 80%;
    height: 80%;
    top: 10%;
    left: 10%;
}

.spinner-segment:nth-child(3) {
    border-top-color: #ff00ff;
    animation-delay: 0.3s;
    width: 60%;
    height: 60%;
    top: 20%;
    left: 20%;
}

.spinner-segment:nth-child(4) {
    border-top-color: #ffaa00;
    animation-delay: 0.45s;
    width: 40%;
    height: 40%;
    top: 30%;
    left: 30%;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

.loading-text {
    color: #00ffff;
    font-size: 11px;
    letter-spacing: 2px;
    animation: text-pulse 1.5s ease-in-out infinite;
}

@keyframes text-pulse {
    0%,
    100% {
        opacity: 0.5;
    }
    50% {
        opacity: 1;
    }
}

/* Load More */
.load-more-row {
    padding: 20px 16px;
    cursor: pointer;
    transition: all 0.2s ease;
}

.load-more-row:hover {
    background: #00ffff08;
}

.load-more-content {
    display: flex;
    align-items: center;
    gap: 12px;
    justify-content: center;
}

.load-more-text {
    color: #00ffff;
    font-size: 11px;
    letter-spacing: 2px;
    text-shadow: 0 0 8px #00ffff66;
}

.load-more-line {
    flex: 1;
    height: 1px;
    background: linear-gradient(90deg, transparent, #00ffff44, transparent);
    max-width: 200px;
}

/* End Row */
.end-row {
    padding: 24px 16px;
}

.end-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
}

.end-line {
    color: #00ffff33;
    font-size: 10px;
    letter-spacing: 4px;
}

.end-text {
    color: #00ff88;
    font-size: 10px;
    letter-spacing: 4px;
    text-shadow: 0 0 8px #00ff8866;
}

/* Empty Row */
.empty-row {
    padding: 48px 16px;
}

.empty-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
}

.empty-icon {
    color: #333;
    font-size: 48px;
    animation: float 3s ease-in-out infinite;
}

@keyframes float {
    0%,
    100% {
        transform: translateY(0);
    }
    50% {
        transform: translateY(-10px);
    }
}

.empty-text {
    color: #444;
    font-size: 12px;
    letter-spacing: 2px;
}

.empty-hint {
    color: #333;
    font-size: 10px;
}

/* Console Footer */
.console-footer {
    position: relative;
    z-index: 5;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    background: linear-gradient(180deg, #0a0a12 0%, #0f0f1a 100%);
    border-top: 1px solid #1a1a2e;
}

.footer-section {
    display: flex;
    align-items: center;
    gap: 8px;
}

.footer-label {
    color: #444;
    font-size: 10px;
    letter-spacing: 1px;
}

.footer-value {
    color: #00ffff;
    font-size: 11px;
    font-weight: 600;
    text-shadow: 0 0 6px #00ffff44;
}

.footer-timestamp {
    color: #00ff88;
    font-size: 10px;
    font-family: "SF Mono", monospace;
    text-shadow: 0 0 6px #00ff8844;
}

/* =========================================
   Light Theme Styles (Tatooine Outpost)
   ========================================= */
.light-theme .hosts-console-container {
    background: 
        linear-gradient(135deg, rgba(245, 230, 211, 0.9) 0%, rgba(239, 224, 204, 0.95) 50%, rgba(232, 212, 188, 0.9) 100%),
        repeating-linear-gradient(
            45deg,
            transparent,
            transparent 10px,
            rgba(184, 134, 11, 0.02) 10px,
            rgba(184, 134, 11, 0.02) 20px
        );
    border: 1px solid #b8860b;
    box-shadow: 
        inset 0 0 100px rgba(184, 134, 11, 0.05),
        0 4px 20px rgba(107, 68, 35, 0.15);
}

.light-theme .console-toolbar {
    background: #e8d4bc;
    border-bottom-color: rgba(184, 134, 11, 0.3);
}

.light-theme .input-wrapper {
    background: #faf3e8;
    border-color: #b8860b;
}

.light-theme .input-prompt {
    color: #cd7f32;
    text-shadow: 0 0 6px rgba(205, 127, 50, 0.25);
}

.light-theme .terminal-input {
    color: #3d2914;
}

.light-theme .terminal-input::placeholder {
    color: #8b7355;
}

.light-theme .input-cursor {
    background: #cd7f32;
}

.light-theme .scanlines {
    background: repeating-linear-gradient(
        0deg,
        rgba(107, 68, 35, 0.03),
        rgba(107, 68, 35, 0.03) 1px,
        transparent 1px,
        transparent 2px
    );
}

.light-theme .crt-vignette {
    background: radial-gradient(
        ellipse at center,
        transparent 0%,
        rgba(107, 68, 35, 0.1) 100%
    );
}

.light-theme .title-text {
    color: #cd7f32;
    text-shadow: 0 0 8px rgba(205, 127, 50, 0.3);
}

.light-theme .title-prefix,
.light-theme .title-suffix {
    color: #8b7355;
}

.light-theme .metric-label {
    color: #6b4423;
}

.light-theme .metric-value {
    color: #3d2914;
}

.light-theme .metric-active {
    color: #daa520;
}

.light-theme .console-toolbar {
    background: #e8d4bc;
    border-bottom-color: rgba(184, 134, 11, 0.25);
}

.light-theme .terminal-input {
    background: #faf3e8;
    border-color: #d4a574;
    color: #3d2914;
}

.light-theme .input-prompt {
    color: #cd7f32;
}

.light-theme .console-btn {
    border-color: #cd7f32;
    color: #cd7f32;
}

.light-theme .console-btn:hover {
    background: rgba(205, 127, 50, 0.12);
    box-shadow: 0 2px 8px rgba(205, 127, 50, 0.3);
}

.light-theme .console-btn.btn-primary {
    border-color: #cd7f32;
    color: #ffffff;
    background: #cd7f32;
}

.light-theme .console-btn.btn-primary:hover {
    background: #b8860b;
    box-shadow: 0 2px 8px rgba(184, 134, 11, 0.3);
}

.light-theme .grid-background {
    background-image:
        linear-gradient(rgba(184, 134, 11, 0.08) 1px, transparent 1px),
        linear-gradient(90deg, rgba(184, 134, 11, 0.08) 1px, transparent 1px);
}

.light-theme .table-header-row {
    background: #e8d4bc;
    border-bottom-color: #d4a574;
}

.light-theme .cell-label {
    color: #cd7f32;
}

.light-theme .cell-divider {
    color: #d4a574;
}

.light-theme .table-row {
    border-bottom-color: #f5e6d3;
}

.light-theme .table-row:hover {
    background: rgba(205, 127, 50, 0.08);
}

.light-theme .domain-text {
    color: #b8860b;
}

.light-theme .ip-text {
    color: #cd7f32;
}

.light-theme .status-enabled {
    background: #daa520;
    box-shadow: 0 0 8px rgba(218, 165, 32, 0.4);
}

.light-theme .status-disabled {
    background: #d4a574;
    box-shadow: none;
}

.light-theme .no-data-message {
    color: #8b7355;
}

.light-theme .table-wrapper::-webkit-scrollbar-track {
    background: #e8d4bc;
}

.light-theme .table-wrapper::-webkit-scrollbar-thumb {
    background: rgba(184, 134, 11, 0.4);
}

.light-theme .table-wrapper::-webkit-scrollbar-thumb:hover {
    background: rgba(184, 134, 11, 0.6);
}

.light-theme .row-index {
    color: #9a7b5a;
}

.light-theme .ip-address {
    color: #cd7f32;
    text-shadow: 0 0 6px rgba(205, 127, 50, 0.25);
}

.light-theme .domain-name {
    color: #b8860b;
    text-shadow: 0 0 6px rgba(184, 134, 11, 0.25);
}

.light-theme .status-text {
    color: #8b7355;
}

.light-theme .status-active .status-text {
    color: #daa520;
    text-shadow: 0 0 6px rgba(218, 165, 32, 0.3);
}

.light-theme .footer-label {
    color: #8b7355;
}

.light-theme .footer-value {
    color: #cd7f32;
    text-shadow: 0 0 6px rgba(205, 127, 50, 0.25);
}

.light-theme .table-row:hover::before {
    background: #cd7f32;
    box-shadow: 0 0 8px rgba(205, 127, 50, 0.4);
}

.light-theme .row-highlight {
    background: rgba(184, 134, 11, 0.05);
}

.light-theme .status-toggle:hover {
    background: rgba(205, 127, 50, 0.08);
}

.light-theme .status-core {
    background: #faf3e8;
    border-color: #d4a574;
}

.light-theme .status-inner {
    background: #b8860b;
}

.light-theme .status-active .status-core {
    border-color: #daa520;
    box-shadow: 0 0 8px rgba(218, 165, 32, 0.4);
}

.light-theme .status-active .status-inner {
    background: #daa520;
}

.light-theme .status-text {
    color: #9a7b5a;
}

.light-theme .status-active .status-text {
    color: #daa520;
    text-shadow: 0 0 6px rgba(218, 165, 32, 0.3);
}

.light-theme .icon-btn {
    border-color: #d4a574;
}

.light-theme .icon-btn:hover {
    border-color: #cd7f32;
    box-shadow: 0 0 8px rgba(205, 127, 50, 0.3);
}

.light-theme .icon-globe {
    color: #cd7f32;
}

.light-theme .icon-edit {
    color: #b8860b;
}

.light-theme .icon-delete {
    color: #b22222;
}

.light-theme .btn-edit:hover {
    border-color: #b8860b;
    box-shadow: 0 0 8px rgba(184, 134, 11, 0.3);
}

.light-theme .btn-delete:hover {
    border-color: #b22222;
    box-shadow: 0 0 8px rgba(178, 34, 34, 0.3);
}

/* Tatooine Desert Texture */
.light-theme .table-row {
    border-bottom-color: rgba(184, 134, 11, 0.15);
    background: linear-gradient(90deg, rgba(250, 243, 232, 0.5) 0%, transparent 100%);
}

.light-theme .table-row:hover {
    background: linear-gradient(90deg, rgba(205, 127, 50, 0.12) 0%, rgba(205, 127, 50, 0.05) 100%);
}

.light-theme .row-highlight {
    background: linear-gradient(90deg, rgba(184, 134, 11, 0.06) 0%, rgba(184, 134, 11, 0.02) 100%);
}

.light-theme .table-header-row {
    background: linear-gradient(180deg, #dcc8a8 0%, #e8d4bc 100%);
    border-bottom: 2px solid #b8860b;
}

.light-theme .console-footer {
    background: linear-gradient(180deg, #e8d4bc 0%, #dcc8a8 100%);
    border-top: 2px solid #b8860b;
}

.light-theme .loading-spinner .spinner-segment {
    border-color: rgba(184, 134, 11, 0.2);
    border-top-color: #cd7f32;
}

.light-theme .loading-text {
    color: #8b7355;
}

.light-theme .end-line {
    color: #d4a574;
}

.light-theme .end-text {
    color: #8b7355;
}

.light-theme .load-more-text {
    color: #cd7f32;
}

.light-theme .load-more-line {
    background: linear-gradient(90deg, transparent, #d4a574, transparent);
}

.light-theme .empty-icon {
    color: #d4a574;
}

.light-theme .empty-text {
    color: #6b4423;
}

.light-theme .empty-hint {
    color: #8b7355;
}
</style>
