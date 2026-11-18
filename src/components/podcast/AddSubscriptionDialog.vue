<template>
  <v-dialog
    :model-value="modelValue"
    max-width="560"
    persistent
    @update:model-value="v => !v && requestClose()"
  >
    <v-card class="add-sub-dialog">
      <div class="dialog-decorator"></div>

      <v-card-title class="console-title-bar">
        <span class="dialog-icon">◈</span>
        <span class="dialog-title">{{ $t('podcast.addSubscription') }}</span>
        <v-spacer />
        <button
          type="button"
          class="close-btn"
          :aria-label="$t('common.close')"
          :disabled="resolving"
          @click="requestClose"
        >
          <v-icon :icon="mdiClose" size="16" aria-hidden="true" />
        </button>
      </v-card-title>

      <v-card-text class="dialog-body">
        <!-- 分段切换：粘贴链接 / OPML 文件 -->
        <div class="segmented" role="tablist">
          <button
            v-for="opt in modes"
            :key="opt.value"
            type="button"
            role="tab"
            class="segment"
            :class="{ active: mode === opt.value }"
            :aria-selected="mode === opt.value"
            @click="switchMode(opt.value)"
          >
            <v-icon :icon="opt.icon" size="15" aria-hidden="true" />
            <span>{{ opt.label }}</span>
          </button>
        </div>

        <!-- 粘贴链接模式 -->
        <div v-if="mode === 'paste'" class="paste-pane">
          <label class="field-label" for="feed-url-input">
            {{ $t('podcast.feedUrlLabel') }}
          </label>
          <div class="input-wrapper url-input" :class="{ 'has-error': urlError }">
            <span class="input-prefix">URL</span>
            <input
              id="feed-url-input"
              ref="urlInputRef"
              v-model="feedUrl"
              type="text"
              class="console-input with-prefix mono"
              :placeholder="$t('podcast.feedUrlPlaceholder')"
              autocomplete="off"
              spellcheck="false"
              :disabled="resolving"
              @keydown.enter.prevent="resolveNow"
              @input="onUrlInput"
            />
          </div>

          <!-- 状态行 -->
          <div v-if="statusLine" class="status-line" :class="statusKind">
            <v-icon v-if="statusKind === 'ok'" :icon="mdiCheck" size="13" aria-hidden="true" />
            <v-icon v-else-if="statusKind === 'loading'" :icon="mdiLoading" size="13" class="spinning" aria-hidden="true" />
            <v-icon v-else :icon="mdiAlertCircleOutline" size="13" aria-hidden="true" />
            <span>{{ statusLine }}</span>
          </div>

          <!-- 预览卡 / skeleton -->
          <div v-if="resolving" class="preview-card skeleton">
            <div class="skel cover"></div>
            <div class="skel-lines">
              <div class="skel l1"></div>
              <div class="skel l2"></div>
              <div class="skel l3"></div>
            </div>
          </div>
          <div v-else-if="resolvedSub" class="preview-card">
            <v-img
              :src="resolvedSub.image_url"
              width="76"
              height="76"
              cover
              class="preview-cover"
              alt=""
            >
              <template #placeholder>
                <div class="skel cover-small"></div>
              </template>
            </v-img>
            <div class="preview-info">
              <div class="preview-title">{{ resolvedSub.title }}</div>
              <div class="preview-meta">
                <span v-if="resolvedSub.author">{{ resolvedSub.author }}</span>
                <span v-if="resolvedSub.author && resolvedSub.pay_type !== 'FREE'" class="dot">·</span>
                <span v-if="resolvedSub.pay_type !== 'FREE'" class="paid-badge">
                  {{ $t('podcast.paidEpisodes', { count: resolvedSub.paid_episode_count }) }}
                </span>
              </div>
              <div class="preview-rss mono">{{ shorten(resolvedSub.rss_url, 52) }}</div>
            </div>
            <span class="free-badge" :class="{ paid: resolvedSub.pay_type !== 'FREE' }">
              {{ resolvedSub.pay_type === 'FREE' ? 'FREE' : $t('podcast.paid') }}
            </span>
          </div>

          <!-- 帮助链接 -->
          <div class="help-row">
            <a class="help-link" href="#" @click.prevent="openRssHelp">
              {{ $t('podcast.howToFindRss') }}
            </a>
          </div>
        </div>

        <!-- OPML 文件模式 -->
        <div v-else class="opml-pane">
          <div class="dialog-instruction">{{ $t('podcast.selectOpml') }}</div>
          <div class="input-wrapper">
            <v-file-input
              v-model="opmlFile"
              class="console-file-input"
              :label="$t('podcast.opmlFile')"
              variant="plain"
              density="compact"
              accept=".opml,.xml"
              :error-messages="opmlError"
              hide-details
            />
          </div>
          <OpmlExportGuide />
        </div>
      </v-card-text>

      <v-card-actions class="console-card-actions">
        <v-spacer />
        <button type="button" class="console-btn" :disabled="resolving" @click="requestClose">
          {{ $t('common.cancel') }}
        </button>
        <button
          type="button"
          class="console-btn primary"
          :disabled="!canSubmit"
          @click="submit"
        >
          <span v-if="submitting" class="btn-icon spinning">↻</span>
          <span>{{ submitLabel }}</span>
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import {
  mdiAlertCircleOutline,
  mdiCheck,
  mdiClose,
  mdiFileDocumentOutline,
  mdiLinkVariant,
  mdiLoading,
} from '@mdi/js'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  importOpml,
  type OpmlImportResult,
  type PodcastSubscription,
  resolveSubscription,
} from '@/api/podcast'
import OpmlExportGuide from '@/components/podcast/OpmlExportGuide.vue'

const props = defineProps<{
  modelValue: boolean
  subscriptions: PodcastSubscription[]
  initialUrl?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'resolved', sub: PodcastSubscription): void
  (e: 'imported', result: OpmlImportResult): void
}>()

const { t } = useI18n()

type Mode = 'paste' | 'opml'
const mode = ref<Mode>('paste')
const feedUrl = ref('')
const urlError = ref('')
const resolvedSub = ref<PodcastSubscription | null>(null)
const resolving = ref(false)
const submitting = ref(false)
const opmlFile = ref<File | File[] | null>(null)
const opmlError = ref('')
const urlInputRef = ref<HTMLInputElement | null>(null)

const modes = computed(() => [
  { value: 'paste' as Mode, label: t('podcast.pasteLink'), icon: mdiLinkVariant },
  { value: 'opml' as Mode, label: t('podcast.opmlFileTab'), icon: mdiFileDocumentOutline },
])

const isXiaoyuzhou = (u: string) => u.includes('xiaoyuzhoufm.com')

const statusKind = computed<'ok' | 'loading' | 'error' | ''>(() => {
  if (resolving.value) return 'loading'
  if (urlError.value) return 'error'
  if (resolvedSub.value) return 'ok'
  return ''
})

const statusLine = computed(() => {
  if (resolving.value) return t('podcast.resolvingFeed')
  if (urlError.value) return urlError.value
  if (resolvedSub.value) {
    return isXiaoyuzhou(feedUrl.value.trim())
      ? t('podcast.resolvedFromXiaoyuzhou')
      : t('podcast.feedValid')
  }
  return ''
})

const canSubmit = computed(() => {
  if (submitting.value) return false
  if (mode.value === 'paste') return !!resolvedSub.value
  return !!opmlFile.value
})

const submitLabel = computed(() => {
  if (mode.value === 'paste') {
    return submitting.value ? t('podcast.subscribing') : t('podcast.subscribe')
  }
  return submitting.value ? t('podcast.importing') : t('podcast.import')
})

function switchMode(m: Mode) {
  if (resolving.value) return
  mode.value = m
}

function reset() {
  mode.value = 'paste'
  feedUrl.value = ''
  urlError.value = ''
  resolvedSub.value = null
  resolving.value = false
  submitting.value = false
  opmlFile.value = null
  opmlError.value = ''
}

function requestClose() {
  if (resolving.value || submitting.value) return
  emit('update:modelValue', false)
}

function shorten(u: string, max: number) {
  return u.length > max ? `${u.slice(0, max - 1)}…` : u
}

function onUrlInput() {
  // 编辑即清空上一次校验结果，避免残留错误/预览
  urlError.value = ''
  resolvedSub.value = null
}

async function resolveNow() {
  const url = feedUrl.value.trim()
  if (!url || resolving.value) return

  urlError.value = ''
  resolvedSub.value = null
  resolving.value = true
  try {
    resolvedSub.value = await resolveSubscription(url)
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    urlError.value = isXiaoyuzhou(url)
      ? t('podcast.noRssExposed')
      : t('podcast.resolveFailed', { message: shorten(msg, 80) })
  } finally {
    resolving.value = false
  }
}

// 粘贴链接后自动校验（去掉常见前后空白即触发）
let autoResolveTimer: ReturnType<typeof setTimeout> | null = null
watch(feedUrl, v => {
  if (mode.value !== 'paste') return
  const url = v.trim()
  if (!/^https?:\/\//.test(url)) return
  if (autoResolveTimer) clearTimeout(autoResolveTimer)
  autoResolveTimer = setTimeout(resolveNow, 400)
})

async function submit() {
  if (!canSubmit.value) return

  if (mode.value === 'paste' && resolvedSub.value) {
    const sub = resolvedSub.value
    if (props.subscriptions.some(s => s.rss_url === sub.rss_url)) {
      urlError.value = t('podcast.alreadySubscribed')
      return
    }
    submitting.value = true
    try {
      emit('resolved', sub)
      emit('update:modelValue', false)
    } finally {
      submitting.value = false
    }
    return
  }

  // OPML 模式
  let file: File | null = null
  if (Array.isArray(opmlFile.value)) {
    file = opmlFile.value[0] ?? null
  } else {
    file = opmlFile.value
  }

  if (!file) {
    opmlError.value = t('podcast.opmlFileRequired')
    return
  }
  if (!file.name.endsWith('.opml') && !file.name.endsWith('.xml')) {
    opmlError.value = t('podcast.opmlFileInvalid')
    return
  }

  submitting.value = true
  try {
    const text = await file.text()
    if (text.length === 0) {
      throw new Error(t('podcast.opmlEmpty'))
    }
    const result = await importOpml(text)
    emit('imported', result)
    emit('update:modelValue', false)
  } catch (e: unknown) {
    opmlError.value = e instanceof Error ? e.message : String(e)
  } finally {
    submitting.value = false
  }
}

function openRssHelp() {
  // 打开 OPML 导出指南作为帮助入口（后续可换成专门的 RSS 查找指南）
  // 这里先用 alert 类提示代替，避免引入新依赖
  window.alert(t('podcast.rssHelpHint'))
}

watch(
  () => props.modelValue,
  open => {
    if (open) {
      reset()
      // OPML 失败条目「手动添加」带回的链接：回填并自动校验
      if (props.initialUrl) {
        feedUrl.value = props.initialUrl
      }
      setTimeout(() => urlInputRef.value?.focus(), 50)
    }
  }
)
</script>

<style scoped>
.add-sub-dialog {
  background: var(--bg-elevated) !important;
  border: 1px solid var(--border);
  border-radius: var(--radius-dialog) !important;
  overflow: hidden;
  box-shadow: var(--shadow-dialog);
  position: relative;
}

.dialog-decorator {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent, var(--accent), transparent);
  opacity: 0.6;
  z-index: 1;
}

.dialog-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px 20px 8px !important;
}

/* 分段控件 */
.segmented {
  display: flex;
  gap: 6px;
}

.segment {
  flex: 1;
  min-height: 44px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-family: var(--jedi-font-ui);
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted);
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-control);
  cursor: pointer;
  transition: all var(--motion-fast) var(--ease-out);
}

.segment:hover {
  color: var(--text);
  background: var(--bg-elevated-2);
}

.segment.active {
  color: var(--on-accent);
  background: var(--accent);
  border-color: var(--accent);
}

/* 粘贴模式 */
.paste-pane,
.opml-pane {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.field-label {
  display: flex;
  align-items: center;
  color: rgb(var(--text-rgb) / 0.7);
  font-size: 12px;
  letter-spacing: 1px;
}

.url-input {
  min-height: 44px;
  transition: border-color var(--motion-fast);
}

.url-input.has-error {
  border-color: var(--danger);
}

.mono {
  font-family: var(--jedi-font-mono);
  font-size: 12px;
}

/* 状态行 */
.status-line {
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: var(--jedi-font-mono);
  font-size: 11px;
  line-height: 1.6;
}

.status-line.ok { color: var(--success); }
.status-line.error { color: var(--danger); }
.status-line.loading { color: var(--accent); }

/* 预览卡 */
.preview-card {
  display: flex;
  gap: 14px;
  padding: 14px;
  background: var(--bg);
  border: 1px solid rgb(var(--accent-rgb) / 0.25);
  border-radius: var(--radius-panel);
  align-items: center;
}

.preview-card.skeleton {
  border-color: var(--border);
}

.preview-cover {
  border-radius: 10px;
  border: 1px solid var(--border);
  flex-shrink: 0;
}

.preview-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.preview-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.preview-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-muted);
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
}

.dot { color: var(--text-subtle); }

.paid-badge {
  color: var(--warning);
}

.preview-rss {
  font-size: 11px;
  color: var(--text-subtle);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.free-badge {
  font-family: var(--jedi-font-mono);
  font-size: 10px;
  letter-spacing: 1px;
  color: var(--success);
  border: 1px solid rgb(var(--success-rgb) / 0.35);
  background: rgb(var(--success-rgb) / 0.1);
  border-radius: 4px;
  padding: 4px 8px;
  flex-shrink: 0;
}

.free-badge.paid {
  color: var(--warning);
  border-color: rgb(var(--warning-rgb) / 0.35);
  background: rgb(var(--warning-rgb) / 0.1);
}

/* skeleton 动画 */
.skel {
  background: var(--bg-elevated-2);
  border-radius: 6px;
  animation: skel-pulse 1.4s ease-in-out infinite;
}

.skel.cover { width: 76px; height: 76px; border-radius: 10px; flex-shrink: 0; }
.skel.cover-small { width: 76px; height: 76px; }
.skel-lines { flex: 1; display: flex; flex-direction: column; gap: 10px; }
.skel.l1 { height: 16px; width: 55%; }
.skel.l2 { height: 12px; width: 75%; }
.skel.l3 { height: 11px; width: 90%; }

@keyframes skel-pulse {
  0%, 100% { opacity: 0.45; }
  50% { opacity: 0.9; }
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.help-row {
  display: flex;
  justify-content: flex-start;
}

.help-link {
  font-size: 12px;
  color: var(--accent);
  text-decoration: none;
}

.help-link:hover {
  text-decoration: underline;
}

.close-btn {
  background: none;
  border: none;
  color: rgb(var(--text-rgb) / 0.5);
  cursor: pointer;
  padding: 4px 8px;
  display: grid;
  place-items: center;
}

.close-btn:hover {
  color: var(--danger);
}

.dialog-instruction {
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.8;
}
</style>
