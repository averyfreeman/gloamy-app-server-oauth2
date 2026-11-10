<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useAuthStore } from '@/stores/auth'

type AuthorizationStatus = 'pending' | 'success' | 'error' | 'expired' | 'cancelled'
type DialogFlowStatus = AuthorizationStatus | 'idle'

interface AuthorizationFlow {
  id: string
  status: AuthorizationStatus
  verification_uri?: string
  user_code?: string
  expires_at?: string
  error?: string
}

interface CodexModel {
  id: string
  label: string
}

interface CodexModelsResponse {
  models: CodexModel[]
  source: 'account' | 'catalog' | 'custom'
  warning?: string | null
  selected_model?: string | null
  is_default: boolean
  connected: boolean
}

interface ApplyModelResponse {
  applied: boolean
  is_default: boolean
  selected_model: string
}

const props = defineProps<{
  open: boolean
  connected: boolean
}>()

const emit = defineEmits<{
  close: []
  connected: []
}>()

let disposed = false
const auth = useAuthStore()
const dialogElement = ref<HTMLDialogElement | null>(null)
const titleElement = ref<HTMLElement | null>(null)
const connected = ref(props.connected)
const flowStatus = ref<DialogFlowStatus>('idle')
const flowId = ref<string | null>(null)
const verificationUri = ref<string | null>(null)
const verificationCode = ref<string | null>(null)
const expiresAt = ref<string | null>(null)
const startingAuthorization = ref(false)
const checkingAuthorization = ref(false)
const cancellingAuthorization = ref(false)
const authorizationRequestError = ref('')
const modelInfo = ref<CodexModelsResponse | null>(null)
const selectedModel = ref('')
const customModelId = ref('')
const loadingModels = ref(false)
const modelError = ref('')
const savingModel = ref(false)
const useAsDefault = ref(false)
let pollTimer: ReturnType<typeof setTimeout> | undefined
let pollRequestController: AbortController | null = null
let modelRequestController: AbortController | null = null

const safeVerificationHref = computed(() => {
  if (!verificationUri.value)
    return null

  try {
    const url = new URL(verificationUri.value)
    return url.protocol === 'https:' ? url.href : null
  }
  catch {
    return null
  }
})

const selectedModelId = computed(() => customModelId.value.trim() || selectedModel.value.trim())
const mustChooseDefault = computed(() => Boolean(modelInfo.value && !modelInfo.value.is_default))
const canApplyModel = computed(() => Boolean(
  modelInfo.value?.connected
  && selectedModelId.value
  && !loadingModels.value
  && !savingModel.value
  && (!mustChooseDefault.value || useAsDefault.value),
))

watch(() => props.connected, (value) => {
  connected.value = value
  if (value && props.open)
    void fetchModels()
})

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    void openDialog()
  }
  else {
    closeNativeDialog()
    stopPolling()
    abortDialogRequests()
    const id = flowId.value
    flowId.value = null
    if (id)
      void cancelAuthorization(id)
  }
}, { immediate: true })

function trapDialogTab(event: KeyboardEvent) {
  const dialog = dialogElement.value
  if (event.key !== 'Tab' || disposed || !dialog?.open)
    return
  const controls = Array.from(dialog.querySelectorAll<HTMLElement>(
    'button:not([disabled]), a[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex="0"]',
  )).filter(element => element.getClientRects().length > 0)
  const first = controls[0]
  const last = controls.at(-1)
  const active = document.activeElement
  if (!first || !last)
    return
  if (event.shiftKey && (active === first || !controls.includes(active as HTMLElement))) {
    event.preventDefault()
    last.focus()
  }
  else if (!event.shiftKey && (active === last || !controls.includes(active as HTMLElement))) {
    event.preventDefault()
    first.focus()
  }
}

function restoreDialogFocus() {
  void nextTick(() => {
    const dialog = dialogElement.value
    if (!disposed && document.hasFocus() && dialog?.open && !dialog.contains(document.activeElement))
      titleElement.value?.focus()
  })
}

onMounted(() => document.addEventListener('keydown', trapDialogTab))

function stopPolling() {
  if (pollTimer !== undefined) {
    clearTimeout(pollTimer)
    pollTimer = undefined
  }
}

function abortDialogRequests() {
  pollRequestController?.abort()
  pollRequestController = null
  modelRequestController?.abort()
  modelRequestController = null
  checkingAuthorization.value = false
  loadingModels.value = false
}

function schedulePoll() {
  stopPolling()
  if (disposed || flowStatus.value !== 'pending' || !flowId.value)
    return

  pollTimer = setTimeout(() => {
    pollTimer = undefined
    void inspectAuthorization()
  }, 2000)
}

function closeNativeDialog() {
  const element = dialogElement.value
  if (element?.open)
    element.close()
}

async function openDialog() {
  await nextTick()
  const element = dialogElement.value
  if (element && !element.open)
    element.showModal()
  await nextTick()
  titleElement.value?.focus()

  if (connected.value)
    await fetchModels()
}

function applyAuthorizationFlow(flow: AuthorizationFlow) {
  flowId.value = flow.id
  flowStatus.value = flow.status
  verificationUri.value = flow.verification_uri ?? verificationUri.value
  verificationCode.value = flow.user_code ?? verificationCode.value
  expiresAt.value = flow.expires_at ?? expiresAt.value
  authorizationRequestError.value = ''
  restoreDialogFocus()

  if (flow.status === 'pending') {
    schedulePoll()
    return
  }

  stopPolling()
  flowId.value = null
  if (flow.status === 'success') {
    connected.value = true
    emit('connected')
    void fetchModels()
  }
}

async function startAuthorization() {
  if (startingAuthorization.value)
    return

  startingAuthorization.value = true
  authorizationRequestError.value = ''
  flowStatus.value = 'idle'
  verificationUri.value = null
  verificationCode.value = null
  expiresAt.value = null
  try {
    const flow = await auth.fetchWithAuth<AuthorizationFlow>('/api/integrations/openai-codex/oauth', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({}),
    })
    if (disposed || !props.open) {
      if (flow.status === 'pending')
        await cancelAuthorization(flow.id)
      else if (flow.status === 'success')
        emit('connected')
      return
    }
    applyAuthorizationFlow(flow)
  }
  catch {
    flowStatus.value = 'error'
  }
  finally {
    startingAuthorization.value = false
    restoreDialogFocus()
  }
}

async function inspectAuthorization() {
  const id = flowId.value
  if (disposed || !id || checkingAuthorization.value || !props.open)
    return

  checkingAuthorization.value = true
  authorizationRequestError.value = ''
  const controller = new AbortController()
  pollRequestController = controller
  try {
    const flow = await auth.fetchWithAuth<AuthorizationFlow>(
      `/api/integrations/openai-codex/oauth/${encodeURIComponent(id)}`,
      { signal: controller.signal },
    )
    if (!disposed && props.open && !controller.signal.aborted && flowId.value === id)
      applyAuthorizationFlow(flow)
  }
  catch {
    if (!disposed && props.open && !controller.signal.aborted && flowId.value === id)
      authorizationRequestError.value = 'Could not check authorization status. Try checking again.'
  }
  finally {
    if (pollRequestController === controller) {
      pollRequestController = null
      checkingAuthorization.value = false
    }
    if (!disposed && props.open && flowStatus.value === 'pending')
      schedulePoll()
  }
}

async function cancelAuthorization(id: string) {
  try {
    return await auth.fetchWithAuth<AuthorizationFlow>(`/api/integrations/openai-codex/oauth/${encodeURIComponent(id)}`, {
      method: 'DELETE',
    })
  }
  catch {
    // The caller must not claim cancellation when the daemon cannot be reached.
    return null
  }
}

async function cancelAuthorizationFromDialog() {
  if (cancellingAuthorization.value)
    return

  cancellingAuthorization.value = true
  stopPolling()
  pollRequestController?.abort()
  pollRequestController = null
  const id = flowId.value
  if (id) {
    const result = await cancelAuthorization(id)
    if (disposed)
      return
    if (result) {
      applyAuthorizationFlow(result)
    }
    else {
      authorizationRequestError.value = 'Could not cancel authorization. It may still complete; retry cancellation or check its status.'
      schedulePoll()
    }
  }
  cancellingAuthorization.value = false
  restoreDialogFocus()
}

function closeSetup() {
  stopPolling()
  abortDialogRequests()
  const id = flowId.value
  flowId.value = null
  if (id)
    void cancelAuthorization(id)
  closeNativeDialog()
  emit('close')
}

function handleEscape(event: Event) {
  event.preventDefault()
  closeSetup()
}

function handleBackdropClick(event: MouseEvent) {
  if (event.target === dialogElement.value)
    closeSetup()
}

function formattedExpiry(value: string): string {
  const date = new Date(value)
  if (Number.isNaN(date.getTime()))
    return value
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(date)
}

function modelSourceLabel(source: CodexModelsResponse['source']): string {
  if (source === 'account')
    return 'ChatGPT account catalog'
  if (source === 'catalog')
    return 'curated Codex catalog'
  return 'custom model IDs'
}

async function fetchModels() {
  if (loadingModels.value)
    return

  loadingModels.value = true
  modelError.value = ''
  const controller = new AbortController()
  modelRequestController = controller
  try {
    const response = await auth.fetchWithAuth<CodexModelsResponse>(
      '/api/integrations/openai-codex/models',
      { signal: controller.signal },
    )
    if (disposed || !props.open || controller.signal.aborted)
      return
    modelInfo.value = response
    connected.value = response.connected
    const currentModel = response.selected_model ?? ''
    if (response.models.some(model => model.id === currentModel)) {
      selectedModel.value = currentModel
      customModelId.value = ''
    }
    else {
      selectedModel.value = ''
      customModelId.value = currentModel
    }
    useAsDefault.value = false
  }
  catch {
    if (!disposed && props.open && !controller.signal.aborted)
      modelError.value = 'Could not load Codex models. Try again.'
  }
  finally {
    if (modelRequestController === controller) {
      modelRequestController = null
      loadingModels.value = false
    }
  }
}

async function applySelectedModel() {
  if (!modelInfo.value || !canApplyModel.value)
    return

  savingModel.value = true
  modelError.value = ''
  try {
    const response = await auth.fetchWithAuth<ApplyModelResponse>('/api/integrations/openai-codex/model', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: selectedModelId.value,
        use_as_default: useAsDefault.value,
      }),
    })
    if (!response.applied) {
      modelError.value = 'The selected Codex model was not applied. Try again.'
      return
    }
    modelInfo.value = {
      ...modelInfo.value,
      selected_model: response.selected_model,
      is_default: response.is_default,
      connected: true,
    }
    if (modelInfo.value.models.some(model => model.id === response.selected_model)) {
      selectedModel.value = response.selected_model
      customModelId.value = ''
    }
    else {
      selectedModel.value = ''
      customModelId.value = response.selected_model
    }
    useAsDefault.value = false
  }
  catch (error) {
    modelError.value = error instanceof Error && error.message.includes('Remove the global API URL override')
      ? 'Remove the global API URL override in Configuration before using Codex as default. Your ChatGPT connection remains saved.'
      : 'Could not apply the selected Codex model. Try again.'
  }
  finally {
    savingModel.value = false
    restoreDialogFocus()
  }
}

onBeforeUnmount(() => {
  disposed = true
  document.removeEventListener('keydown', trapDialogTab)
  stopPolling()
  abortDialogRequests()
  const id = flowId.value
  flowId.value = null
  if (id)
    void cancelAuthorization(id)
})
</script>

<template>
  <dialog
    ref="dialogElement"
    aria-labelledby="codex-setup-title"
    class="m-auto w-[min(40rem,calc(100vw-2rem))] max-w-none max-h-[calc(100vh-2rem)] overflow-y-auto rounded-2xl border border-border bg-background p-0 text-foreground shadow-2xl backdrop:bg-black/60"
    @cancel="handleEscape"
    @click="handleBackdropClick"
  >
    <div class="p-6 sm:p-7">
      <div class="flex items-start justify-between gap-4 mb-6">
        <div>
          <p class="text-[11px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            OpenAI Codex
          </p>
          <h2
            id="codex-setup-title"
            ref="titleElement"
            tabindex="-1"
            class="mt-1 text-xl font-semibold tracking-tight outline-none"
          >
            Connect with ChatGPT
          </h2>
          <p class="mt-2 max-w-prose text-sm text-muted-foreground">
            Authorize Gloamy with your ChatGPT account. Your account credentials stay on the server.
          </p>
        </div>
        <button
          type="button"
          aria-label="Close setup"
          class="rounded-lg p-2 text-muted-foreground hover:bg-card hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
          @click="closeSetup"
        >
          <span aria-hidden="true" class="text-xl leading-none">×</span>
        </button>
      </div>

      <p v-if="flowStatus === 'success'" role="status" aria-live="polite" class="mb-4 rounded-xl border border-emerald-500/30 bg-emerald-500/10 px-4 py-3 text-sm text-emerald-700 dark:text-emerald-300">
        ChatGPT account connected.
      </p>

      <section v-if="!connected" aria-live="polite" class="space-y-4">
        <div v-if="flowStatus === 'pending'" class="space-y-4 rounded-xl border border-border/60 bg-card/30 p-4">
          <div class="flex items-center gap-2 text-sm font-medium">
            <span class="size-2 animate-pulse rounded-full bg-sky-500" />
            Waiting for ChatGPT authorization
          </div>
          <p v-if="safeVerificationHref" class="text-sm text-muted-foreground">
            Open
            <a :href="safeVerificationHref" target="_blank" rel="noopener noreferrer" class="font-medium text-primary underline underline-offset-4">
              the ChatGPT verification page
            </a>
            and enter this code:
          </p>
          <p v-else class="text-sm text-muted-foreground">
            Open the verification page and enter this code:
          </p>
          <p class="rounded-lg bg-background px-4 py-3 text-center font-mono text-2xl font-semibold tracking-[0.2em] select-all">
            {{ verificationCode ?? 'Code unavailable' }}
          </p>
          <p v-if="expiresAt" class="text-xs text-muted-foreground">
            This code expires {{ formattedExpiry(expiresAt) }}.
          </p>
          <p v-if="authorizationRequestError" role="alert" class="text-sm text-amber-600 dark:text-amber-400">
            {{ authorizationRequestError }}
          </p>
          <div class="flex flex-wrap gap-2">
            <button
              type="button"
              :disabled="checkingAuthorization"
              class="rounded-lg border border-border px-3 py-2 text-sm font-medium hover:bg-card disabled:cursor-not-allowed disabled:opacity-50"
              @click="inspectAuthorization"
            >
              {{ checkingAuthorization ? 'Checking…' : 'Check status' }}
            </button>
            <button
              type="button"
              :disabled="cancellingAuthorization"
              class="rounded-lg px-3 py-2 text-sm font-medium text-muted-foreground hover:bg-card disabled:opacity-50"
              @click="cancelAuthorizationFromDialog"
            >
              {{ cancellingAuthorization ? 'Cancelling…' : 'Cancel authorization' }}
            </button>
          </div>
        </div>

        <p v-else-if="flowStatus === 'expired'" role="status" class="rounded-xl border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-700 dark:text-amber-300">
          The verification code expired. Start a new authorization to try again.
        </p>
        <p v-else-if="flowStatus === 'cancelled'" role="status" class="rounded-xl border border-border bg-card/30 px-4 py-3 text-sm text-muted-foreground">
          Authorization was cancelled.
        </p>
        <p v-else-if="flowStatus === 'error'" role="alert" class="rounded-xl border border-destructive/30 bg-destructive/5 px-4 py-3 text-sm text-destructive">
          Authorization could not be completed. Start again to retry.
        </p>

        <div class="flex flex-wrap items-center gap-3">
          <button
            v-if="flowStatus !== 'pending'"
            type="button"
            :disabled="startingAuthorization"
            class="rounded-xl bg-primary px-4 py-2.5 text-sm font-semibold text-primary-foreground hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
            @click="startAuthorization"
          >
            {{ startingAuthorization ? 'Starting…' : flowStatus === 'idle' ? 'Connect ChatGPT' : 'Try again' }}
          </button>
          <span class="text-xs text-muted-foreground">No model or default provider changes until you apply one below.</span>
        </div>
      </section>

      <section v-else class="space-y-4" aria-labelledby="codex-model-title">
        <div class="flex items-start justify-between gap-4">
          <div>
            <h3 id="codex-model-title" class="text-base font-semibold">
              Codex model
            </h3>
            <p class="mt-1 text-sm text-muted-foreground">
              Choose a model for Codex. Applying a model is a separate action from connecting your account.
            </p>
          </div>
          <button
            type="button"
            :disabled="loadingModels"
            aria-label="Refresh Codex models"
            class="shrink-0 rounded-lg border border-border px-3 py-2 text-xs font-medium hover:bg-card disabled:opacity-50"
            @click="fetchModels"
          >
            Refresh
          </button>
        </div>

        <p v-if="loadingModels" role="status" class="text-sm text-muted-foreground">
          Loading available Codex models…
        </p>
        <p v-if="modelInfo" class="text-xs text-muted-foreground">
          Model options from the {{ modelSourceLabel(modelInfo.source) }}.
        </p>
        <p v-if="modelInfo?.warning" class="rounded-lg bg-amber-500/10 px-3 py-2 text-xs text-amber-700 dark:text-amber-300">
          {{ modelInfo.warning }}
        </p>
        <p v-if="modelError" role="alert" class="text-sm text-destructive">
          {{ modelError }}
        </p>

        <div v-if="modelInfo?.connected" class="space-y-4">
          <div>
            <label for="codex-model-select" class="mb-1.5 block text-sm font-medium">Available models</label>
            <select
              id="codex-model-select"
              v-model="selectedModel"
              class="w-full rounded-xl border border-border bg-background px-3 py-2.5 text-sm outline-none focus-visible:ring-2 focus-visible:ring-primary"
              @change="customModelId = ''"
            >
              <option value="">
                Choose a curated model
              </option>
              <option v-for="model in modelInfo.models" :key="model.id" :value="model.id">
                {{ model.label }}
              </option>
            </select>
          </div>

          <div>
            <label for="codex-custom-model" class="mb-1.5 block text-sm font-medium">Custom model ID</label>
            <input
              id="codex-custom-model"
              v-model="customModelId"
              type="text"
              autocomplete="off"
              spellcheck="false"
              placeholder="Enter a Codex model ID"
              class="w-full rounded-xl border border-border bg-background px-3 py-2.5 font-mono text-sm outline-none placeholder:font-sans placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-primary"
              @input="selectedModel = ''"
            >
          </div>

          <p v-if="modelInfo.is_default" class="text-xs text-muted-foreground">
            Codex is the current default provider. Applying updates its default model.
          </p>
          <label v-else class="flex cursor-pointer items-start gap-2.5 rounded-xl border border-border/60 bg-card/20 p-3 text-sm">
            <input v-model="useAsDefault" type="checkbox" class="mt-0.5 accent-primary">
            <span>
              <span class="block font-medium">Use Codex as default</span>
              <span class="mt-0.5 block text-xs text-muted-foreground">This changes the global default provider and model.</span>
            </span>
          </label>

          <button
            type="button"
            :disabled="!canApplyModel"
            class="rounded-xl bg-primary px-4 py-2.5 text-sm font-semibold text-primary-foreground hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
            @click="applySelectedModel"
          >
            {{ savingModel ? 'Applying…' : 'Apply model' }}
          </button>
        </div>
      </section>

      <div class="mt-7 flex justify-end border-t border-border/50 pt-4">
        <button
          type="button"
          class="rounded-lg px-3 py-2 text-sm font-medium text-muted-foreground hover:bg-card hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
          @click="closeSetup"
        >
          Close
        </button>
      </div>
    </div>
  </dialog>
</template>
