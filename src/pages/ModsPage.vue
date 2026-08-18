<template>
  <div class="explore-page" :class="{ 'explore-page--split': !!modpackPickerHit }">
  <div class="explore-main">

    <!-- Search + category pills -->
    <div class="search-tab-row">
      <div class="search-bar">
        <input
          v-model="searchInput"
          class="search-input"
          :placeholder="$t('mods.searchPlaceholder', { tab: tabs.find(tab => tab.key === activeTab)?.label?.toLowerCase() ?? activeTab })"
          @keyup.enter="triggerSearch"
        />
      </div>

      <button
        v-for="tab in visibleTabs"
        :key="tab.key"
        class="tab-pill"
        :class="{ active: activeTab === tab.key }"
        @click="switchTab(tab.key)"
      >{{ tab.label }}</button>
    </div>

    <!-- Filters (hidden on Servers tab) -->
    <div v-if="activeTab !== 'servers'" class="controls-row">
      <select
        v-if="activeTab !== 'modpacks'"
        v-model="filterSource"
        class="filter-select"
        @change="doSearch"
      >
        <option value="modrinth">Modrinth</option>
        <option value="curseforge">CurseForge</option>
        <option value="both">Modrinth + CurseForge</option>
      </select>

      <select v-model="filterVersion" class="filter-select" @change="doSearch">
        <option value="">{{ $t('mods.filters.allVersions') }}</option>
        <option v-for="v in releaseVersions" :key="v" :value="v">{{ v }}</option>
      </select>

      <select
        v-if="showLoaderFilter"
        v-model="filterLoader"
        class="filter-select"
        @change="doSearch"
      >
        <option value="">{{ $t('mods.filters.allLoaders') }}</option>
        <option value="fabric">{{ $t('mods.loaders.fabric') }}</option>
        <option value="forge">{{ $t('mods.loaders.forge') }}</option>
        <option value="neoforge">{{ $t('mods.loaders.neoforge') }}</option>
        <option value="quilt">{{ $t('mods.loaders.quilt') }}</option>
      </select>

      <select v-model="filterSort" class="filter-select" @change="doSearch">
        <option value="relevance">Relevance</option>
        <option value="downloads">Most downloads</option>
        <option value="newest">Newest</option>
        <option value="updated">Recently updated</option>
      </select>

      <div class="view-toggle">
        <button
          class="view-toggle-btn"
          :class="{ active: viewMode === 'list' }"
          title="List view"
          @click="viewMode = 'list'"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="1" y="2" width="12" height="2" fill="currentColor"/>
            <rect x="1" y="6" width="12" height="2" fill="currentColor"/>
            <rect x="1" y="10" width="12" height="2" fill="currentColor"/>
          </svg>
        </button>
        <button
          class="view-toggle-btn"
          :class="{ active: viewMode === 'grid' }"
          title="Grid view"
          @click="viewMode = 'grid'"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="1" y="1" width="5" height="5" fill="currentColor"/>
            <rect x="8" y="1" width="5" height="5" fill="currentColor"/>
            <rect x="1" y="8" width="5" height="5" fill="currentColor"/>
            <rect x="8" y="8" width="5" height="5" fill="currentColor"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Active category chips -->
    <TransitionGroup v-if="filterCategories.length && activeTab !== 'Servers'" name="chip-pop" tag="div" class="chip-row">
      <button v-for="c in filterCategories" :key="c" class="chip" @click="removeCategory(c)">{{ c }} ×</button>
      <button key="__clear" class="chip chip--clear" @click="clearCategories">{{ $t('mods.clearAll') }}</button>
    </TransitionGroup>

    <div class="explore-body">

    <!-- Categories sidebar (pixel-sampled from Figma, 2026-08-14) -->
    <aside v-if="activeTab !== 'servers'" class="categories-sidebar">
      <p class="categories-title">{{ $t('mods.categories') }}</p>
      <button
        v-for="cat in visibleCategories"
        :key="cat.name"
        class="category-row"
        :class="{ active: filterCategories.includes(cat.name) }"
        @click="toggleCategory(cat.name); doSearch()"
      >{{ cat.name }}</button>
    </aside>

    <Transition name="tab-fade" mode="out-in">
    <!-- ── Servers tab ──────────────────────────────────────────────────────── -->
    <div v-if="activeTab === 'servers'" key="servers" class="servers-area">

      <!-- Top bar -->
      <div class="servers-topbar">
        <button class="server-action-btn" @click="showAddForm = !showAddForm">
          {{ showAddForm ? $t('mods.server.cancelServer') : $t('mods.server.addServer') }}
        </button>
        <button class="server-action-btn" :disabled="serversLoading" @click="refreshServers">
          <span v-if="serversLoading" class="spinner sm" />
          <template v-else>{{ $t('mods.server.refresh') }}</template>
        </button>
      </div>

      <!-- Add server form -->
      <Transition name="add-form">
        <div v-if="showAddForm" class="add-server-form">
          <input
            v-model="newHost"
            class="server-input"
            :placeholder="$t('mods.server.ipPlaceholder')"
            @keyup.enter="submitAddServer"
          />
          <input
            v-model.number="newPort"
            class="server-input server-input--port"
            type="number"
            placeholder="25565"
            min="1"
            max="65535"
          />
          <input
            v-model="newName"
            class="server-input"
            :placeholder="$t('mods.server.namePlaceholder')"
            @keyup.enter="submitAddServer"
          />
          <button class="server-add-confirm" :disabled="!newHost || addingServer" @click="submitAddServer">
            <span v-if="addingServer" class="spinner sm" />
            <template v-else>{{ $t('mods.server.add') }}</template>
          </button>
        </div>
      </Transition>

      <!-- Server list -->
      <div v-if="serversLoading && !servers.length" class="skeleton-list">
        <div v-for="i in 6" :key="i" class="row-skeleton">
          <Skeleton variant="block" width="56px" height="56px" radius="4px" />
          <div class="row-skeleton-lines">
            <Skeleton variant="text" width="35%" />
            <Skeleton variant="text" width="70%" />
          </div>
        </div>
      </div>

      <div v-else-if="serversError" class="state-area">
        <span class="state-text error-text">{{ serversError }}</span>
      </div>

      <div v-else class="server-list">
        <div
          v-for="s in servers"
          :key="s.id"
          class="server-card"
          :class="{ offline: !s.online }"
        >
          <!-- Favicon -->
          <div class="server-favicon-wrap">
            <img v-if="s.favicon" :src="s.favicon" class="server-favicon" :alt="s.name" />
            <div v-else class="server-favicon-fallback">{{ s.name[0] }}</div>
          </div>

          <!-- Info -->
          <div class="server-info">
            <div class="server-name-row">
              <span class="server-name">{{ s.name }}</span>
              <span v-if="s.featured" class="server-badge">{{ $t('mods.server.featured') }}</span>
              <span v-if="!s.online && serversLoading" class="server-pinging-badge">{{ $t('mods.server.pinging') }}</span>
            <span v-else-if="!s.online" class="server-offline-badge">{{ $t('mods.server.offline') }}</span>
            </div>
            <p v-if="s.motd" class="server-motd">{{ s.motd }}</p>
            <span class="server-ip">{{ s.host }}{{ s.port !== 25565 ? `:${s.port}` : '' }}</span>
          </div>

          <!-- Stats -->
          <div class="server-stats">
            <div v-if="s.online" class="stat-row">
              <span class="stat-label">{{ $t('mods.server.ping') }}</span>
              <span class="stat-value" :class="pingClass(s.ping)">{{ s.ping }}ms</span>
            </div>
            <div v-if="s.online" class="stat-row">
              <span class="stat-label">{{ $t('mods.server.players') }}</span>
              <span class="stat-value">{{ s.playersOnline }}/{{ s.playersMax }}</span>
            </div>
            <div v-if="s.online && s.version" class="stat-row">
              <span class="stat-label">{{ $t('mods.server.version') }}</span>
              <span class="stat-value version-val">{{ s.version }}</span>
            </div>
          </div>

          <!-- Actions -->
          <div class="server-actions">
            <button
              v-if="!s.featured"
              class="server-remove-btn"
              title="Remove server"
              @click="deleteServer(s.id)"
            >✕</button>
            <button
              v-if="s.online"
              class="install-btn"
              @click="openServerPicker(s, $event)"
            >{{ $t('mods.server.addToProfile') }}</button>
          </div>
        </div>
      </div>

    </div>

    <!-- ── Content list (non-server tabs) ─────────────────────────────────── -->
    <div
      v-else
      :key="activeTab"
      ref="listEl"
      class="content-list"
      :class="{ 'drag-over': contentDragOver, 'content-list--grid': viewMode === 'grid' }"
      @dragover.prevent="contentDragOver = true"
      @dragleave="contentDragOver = false"
      @drop.prevent="onContentDrop"
    >
      <div v-if="contentDragOver" class="drop-hint">Drop a .jar to install it to your active profile</div>

      <div v-if="loading && !results.length" class="skeleton-list">
        <div v-for="i in 6" :key="i" class="row-skeleton">
          <Skeleton variant="block" width="72px" height="72px" radius="4px" />
          <div class="row-skeleton-lines">
            <Skeleton variant="text" width="35%" />
            <Skeleton variant="text" width="85%" />
            <Skeleton variant="text" width="55%" />
          </div>
        </div>
      </div>

      <div v-else-if="error" class="state-area">
        <div class="state-stack">
          <Icon name="warning" :size="28" class="state-icon" />
          <span class="state-text error-text">{{ error }}</span>
          <button class="retry-btn" @mouseenter="playHover" @click="doSearch">{{ $t('mods.retry') }}</button>
        </div>
      </div>

      <div v-else-if="!loading && !results.length" class="state-area">
        <div class="state-stack">
          <Icon name="search" :size="28" class="state-icon" />
          <span class="state-text">{{ $t('mods.noResults') }}</span>
          <button class="retry-btn" @mouseenter="playHover" @click="doSearch">{{ $t('mods.retry') }}</button>
        </div>
      </div>

      <template v-else>
        <TransitionGroup name="row-stagger">
        <div
          v-for="(hit, idx) in results"
          :key="`${hit.source}-${hit.id}`"
          class="mod-row"
          :style="{ '--i': idx % 12 }"
          @contextmenu="openModMenu($event, hit)"
        >

          <span
            class="mod-on-pill"
            :class="{ 'mod-on-pill--on': installedMap.get(`${hit.source}-${hit.id}`)?.size }"
          >
            <span class="mod-on-dot" />
            {{ installedMap.get(`${hit.source}-${hit.id}`)?.size ? $t('mods.on') : $t('mods.off') }}
          </span>

          <div class="mod-icon-wrap">
            <img v-if="hit.iconUrl" :src="hit.iconUrl" class="mod-icon" :alt="hit.title" />
            <div v-else class="mod-icon-fallback">{{ hit.title[0] }}</div>
          </div>

          <div class="mod-info">
            <div class="mod-name-row">
              <button class="mod-name mod-name-btn" @click="openDetails(hit)">{{ hit.title }}</button>
              <span class="mod-stat">{{ formatNum(hit.downloads) }} ↓</span>
            </div>
            <span v-if="hit.author" class="mod-author">
              by {{ hit.author }}
            </span>
            <p class="mod-desc">{{ hit.description }}</p>
            <div class="mod-tags">
              <span v-for="c in hit.categories.slice(0, 4)" :key="c" class="mod-tag">{{ c }}</span>
            </div>
          </div>

          <div class="install-area">
            <span class="mod-stat mod-stat--footer">{{ formatNum(hit.downloads) }} ↓</span>
            <button class="details-btn" title="View details" @mouseenter="playHover" @click="openDetails(hit)">
              <Icon name="eye" :size="14" />
            </button>
            <button
              class="install-btn"
              :disabled="installingSet.has(`${hit.source}-${hit.id}`)"
              @mouseenter="playHover"
              @click="hit.projectType === 'modpack' ? openModpackPicker(hit) : openModPicker(hit, $event)"
            >
              <span v-if="installingSet.has(`${hit.source}-${hit.id}`)" class="spinner sm" />
              <template v-else>{{ $t('mods.install') }}</template>
            </button>
          </div>

        </div>
        </TransitionGroup>

        <div v-if="hasMore" class="load-more-row">
          <button class="load-more-btn" :disabled="loading" @mouseenter="playHover" @click="loadMore">
            <span v-if="loading" class="spinner sm" />
            <template v-else>{{ $t('mods.loadMore') }}</template>
          </button>
        </div>
      </template>

    </div>
    </Transition>

    </div>

  </div>

  <!-- ── Modpack version panel — slides in beside the explorer instead of a
       floating popover, pushing it left into the freed-up space. ──────────── -->
  <Transition name="side-panel">
    <div v-if="modpackPickerHit" class="version-side-panel">
      <div class="version-panel-header">
        <div class="mod-icon-wrap version-panel-icon-wrap">
          <img v-if="modpackPickerHit.iconUrl" :src="modpackPickerHit.iconUrl" class="mod-icon" :alt="modpackPickerHit.title" />
          <div v-else class="mod-icon-fallback">{{ modpackPickerHit.title[0] }}</div>
        </div>
        <div class="version-panel-header-text">
          <span class="version-panel-name">{{ modpackPickerHit.title }}</span>
          <span class="version-panel-sub">Choose a version</span>
        </div>
      </div>

      <!-- One project can have several pack releases for the same Minecraft
           version (e.g. multiple Fabulously Optimized builds all on 1.21.11) —
           filter by MC version first instead of scrolling through everything. -->
      <div v-if="modpackGameVersions.length > 1" class="version-filter-row">
        <select v-model="modpackVersionFilter" class="version-filter-select">
          <option value="">All versions ({{ modpackVersions.length }})</option>
          <option v-for="gv in modpackGameVersions" :key="gv" :value="gv">{{ gv }}</option>
        </select>
      </div>

      <div v-if="modpackVerLoading" class="picker-empty">Loading versions…</div>
      <div v-else-if="!filteredModpackVersions.length" class="picker-empty">No versions found</div>

      <div class="version-side-list">
        <button
          v-for="v in filteredModpackVersions"
          :key="v.id"
          class="picker-row picker-row--version"
          :disabled="modpackInstalling"
          @click="confirmModpackInstall(v.id)"
        >
          <div class="picker-profile-info">
            <span class="picker-profile-name">{{ v.name || v.version_number }}</span>
            <span class="picker-profile-meta">{{ v.game_versions[0] }} · {{ v.loaders.join(', ') || 'vanilla' }}</span>
          </div>
          <span v-if="modpackInstalling" class="spinner sm" />
        </button>
      </div>

      <div class="picker-footer">
        <button class="picker-btn picker-btn--cancel" @click="closeModpackPicker">{{ $t('mods.picker.cancel') }}</button>
      </div>
    </div>
  </Transition>

  <!-- ── Mod profile picker (portal) ────────────────────────────────────────── -->
  <Teleport to="body">
    <Transition name="picker">
      <div v-if="pickerHit" class="picker-overlay" @click.self="closePicker">
        <div class="picker-panel" :style="pickerPos">

          <p class="picker-title">{{ $t('mods.picker.installTitle') }}</p>
          <p class="picker-sub">{{ pickerHit.title }}</p>

          <div v-if="!profiles.length" class="picker-empty">{{ $t('mods.picker.noProfiles') }}</div>

          <label
            v-for="p in profiles"
            :key="p.id"
            class="picker-row"
            :class="{
              checked:   pickerSelected.includes(p.id),
              installed: pickerHit && installedMap.get(`${pickerHit.source}-${pickerHit.id}`)?.has(p.id),
            }"
          >
            <template v-if="!(pickerHit && installedMap.get(`${pickerHit.source}-${pickerHit.id}`)?.has(p.id))">
              <input type="checkbox" :value="p.id" v-model="pickerSelected" class="picker-check" />
            </template>
            <div class="picker-profile-info">
              <span class="picker-profile-name">{{ p.name }}</span>
              <span class="picker-profile-meta">{{ p.version }} · {{ p.loader }}</span>
            </div>
            <span v-if="pickerHit && installedMap.get(`${pickerHit.source}-${pickerHit.id}`)?.has(p.id)" class="picker-installed-label">{{ $t('mods.picker.installed') }}</span>
          </label>

          <div class="picker-footer">
            <button class="picker-btn picker-btn--cancel" @click="closePicker">{{ $t('mods.picker.cancel') }}</button>
            <button
              class="picker-btn picker-btn--confirm"
              :disabled="!pickerSelected.length || pickerInstalling"
              @click="confirmModInstall"
            >
              <span v-if="pickerInstalling" class="spinner sm" />
              <template v-else>{{ $t('mods.picker.installBtn', { count: pickerSelected.length }) }}</template>
            </button>
          </div>

        </div>
      </div>
    </Transition>
  </Teleport>

  <!-- ── Server profile picker (portal) ─────────────────────────────────────── -->
  <Teleport to="body">
    <Transition name="picker">
      <div v-if="serverPickerServer" class="picker-overlay" @click.self="closeServerPicker">
        <div class="picker-panel" :style="pickerPos">

          <p class="picker-title">{{ $t('mods.picker.addTitle') }}</p>
          <p class="picker-sub">{{ serverPickerServer.name }}</p>

          <div v-if="!profiles.length" class="picker-empty">{{ $t('mods.picker.noProfiles') }}</div>

          <label
            v-for="p in profiles"
            :key="p.id"
            class="picker-row"
            :class="{
              checked:   serverPickerSelected.includes(p.id),
              installed: serverPickerServer && serverAddedMap.get(serverKey(serverPickerServer))?.has(p.id),
            }"
          >
            <template v-if="!(serverPickerServer && serverAddedMap.get(serverKey(serverPickerServer))?.has(p.id))">
              <input type="checkbox" :value="p.id" v-model="serverPickerSelected" class="picker-check" />
            </template>
            <div class="picker-profile-info">
              <span class="picker-profile-name">{{ p.name }}</span>
              <span class="picker-profile-meta">{{ p.version }} · {{ p.loader }}</span>
            </div>
            <span v-if="serverPickerServer && serverAddedMap.get(serverKey(serverPickerServer))?.has(p.id)" class="picker-installed-label">{{ $t('mods.picker.added') }}</span>
          </label>

          <div class="picker-footer">
            <button class="picker-btn picker-btn--cancel" @click="closeServerPicker">{{ $t('mods.picker.cancel') }}</button>
            <button
              class="picker-btn picker-btn--confirm"
              :disabled="!serverPickerSelected.length || serverPickerInstalling"
              @click="confirmServerAdd"
            >
              <span v-if="serverPickerInstalling" class="spinner sm" />
              <template v-else>{{ $t('mods.picker.addBtn', { count: serverPickerSelected.length }) }}</template>
            </button>
          </div>

        </div>
      </div>
    </Transition>
  </Teleport>

  <!-- ── Mod details panel ────────────────────────────────────────────────── -->
  <Teleport to="body">
    <Transition name="modal-fade">
      <div v-if="detailsHit" class="details-overlay" @click.self="closeDetails">
        <div class="details-modal">

          <button class="details-close" title="Close" @click="closeDetails">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <line x1="1" y1="1" x2="13" y2="13" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
              <line x1="13" y1="1" x2="1" y2="13" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
            </svg>
          </button>

          <div class="details-header">
            <div class="mod-icon-wrap details-icon-wrap">
              <img v-if="detailsHit.iconUrl" :src="detailsHit.iconUrl" class="mod-icon" :alt="detailsHit.title" />
              <div v-else class="mod-icon-fallback">{{ detailsHit.title[0] }}</div>
            </div>
            <div class="details-header-text">
              <span class="details-title">{{ detailsHit.title }}</span>
              <span class="details-sub">
                <span v-if="detailsData?.author">by {{ detailsData.author }}</span>
                <span v-if="detailsData?.author" class="sep">&middot;</span>
                <span class="details-source-badge">{{ detailsHit.source === 'modrinth' ? 'Modrinth' : 'CurseForge' }}</span>
              </span>
            </div>
          </div>

          <div class="details-body">
            <div v-if="detailsLoading" class="picker-empty">Loading…</div>
            <div v-else-if="detailsError" class="state-stack">
              <Icon name="warning" :size="24" class="state-icon" />
              <span class="state-text error-text">{{ detailsError }}</span>
            </div>
            <template v-else-if="detailsData">
              <div class="details-stats-row">
                <span class="details-stat">{{ formatNum(detailsData.downloads) }} downloads</span>
                <span v-if="detailsData.license" class="details-stat">{{ detailsData.license }}</span>
                <span v-if="detailsData.updatedAt" class="details-stat">Updated {{ new Date(detailsData.updatedAt).toLocaleDateString() }}</span>
              </div>

              <div v-if="detailsData.categories.length" class="mod-tags details-tags">
                <span v-for="c in detailsData.categories" :key="c" class="mod-tag">{{ c }}</span>
              </div>

              <div v-if="detailsData.gallery.length" class="details-gallery">
                <img v-for="(g, i) in detailsData.gallery" :key="i" :src="g" class="details-gallery-img" loading="lazy" />
              </div>

              <div v-if="detailsHtml" class="details-description" v-html="detailsHtml" />
              <p v-else class="mod-desc details-no-desc">No description provided.</p>
            </template>
          </div>

          <div class="details-footer">
            <button class="picker-btn picker-btn--cancel" @click="openSourceUrl(detailsHit)">Open on {{ detailsHit.source === 'modrinth' ? 'Modrinth' : 'CurseForge' }}</button>
            <button
              class="picker-btn picker-btn--confirm"
              @click="installFromDetails(detailsHit, $event)"
            >{{ $t('mods.install') }}</button>
          </div>

        </div>
      </div>
    </Transition>
  </Teleport>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import type { ExploreHit, ExploreSource, ModDetails, ModrinthProjectType, LaunchProfile, ServerStatus, ModrinthVersion } from '../types/index'
import { showToast } from '../composables/useToasts'
import { openContextMenu } from '../composables/useContextMenu'
import { playHover } from '../composables/useSounds'
import Skeleton from '../components/common/Skeleton.vue'
import Icon from '../components/common/Icon.vue'

// ── Tab config ────────────────────────────────────────────────────────────────

const { t } = useI18n()

const tabs = computed(() => [
  { key: 'mods',          label: t('mods.tabs.mods'),          type: 'mod'         as ModrinthProjectType },
  { key: 'modpacks',      label: t('mods.tabs.modpacks'),      type: 'modpack'      as ModrinthProjectType },
  { key: 'shaders',       label: t('mods.tabs.shaders'),       type: 'shader'       as ModrinthProjectType },
  { key: 'resourcepacks', label: t('mods.tabs.resourcepacks'), type: 'resourcepack' as ModrinthProjectType },
  { key: 'servers',       label: t('mods.tabs.servers'),       type: null },
  { key: 'datapacks',     label: t('mods.tabs.datapacks'),     type: 'datapack'     as ModrinthProjectType },
])

// Figma redesign (2026-08-14): Servers moves to the end of the pill row
// (design order is Mods/Modpacks/Shaders/Resourcepacks/Datapacks/Servers,
// not the tabs array's original order).
const visibleTabs = computed(() => {
  const servers = tabs.value.filter(tab => tab.key === 'servers')
  const rest    = tabs.value.filter(tab => tab.key !== 'servers')
  return [...rest, ...servers]
})

const activeTab  = ref<string>('mods')
const activeType = computed(() => tabs.value.find(t => t.key === activeTab.value)?.type ?? null)

// ── List / grid view toggle ──────────────────────────────────────────────────
const VIEW_MODE_KEY = 'mods.viewMode'
const viewMode = ref<'list' | 'grid'>(localStorage.getItem(VIEW_MODE_KEY) === 'grid' ? 'grid' : 'list')
watch(viewMode, v => localStorage.setItem(VIEW_MODE_KEY, v))

const showLoaderFilter = computed(() =>
  ['mods', 'modpacks', 'datapacks'].includes(activeTab.value)
)

// ── Filters ───────────────────────────────────────────────────────────────────

const filterVersion    = ref('')
const filterLoader     = ref('')
const filterCategories = ref<string[]>([])
const filterSource     = ref<ExploreSource>('modrinth')
const filterSort       = ref('relevance')

// CurseForge modpacks download as a plain .zip with a different install pipeline
// (overrides folder, no mrpack unpacking) that the backend doesn't implement — the
// modpack tab's install flow (openModpackPicker → modrinth.versions) is Modrinth-only,
// so force that source regardless of the picker to avoid a broken install.
const effectiveSource = computed<ExploreSource>(() => activeType.value === 'modpack' ? 'modrinth' : filterSource.value)

const releaseVersions     = ref<string[]>([])
const availableCategories = ref<{ name: string; project_type: string }[]>([])

const visibleCategories = computed(() => {
  const type = activeType.value
  if (!type) return []
  // Modrinth's "cursed" category (joke/broken mods) isn't what most people mean by
  // "browse horror mods" — swap it for a synthetic entry that runs a keyword search
  // for "horror" instead of a category facet (Modrinth has no real "horror" tag).
  const real = availableCategories.value.filter(c => c.project_type === type && c.name !== 'cursed')
  return type === 'mod' ? [...real, { name: 'horror', project_type: type }] : real
})

// Modrinth/CurseForge don't expose real "categories:horror" or "categories:cursed"
// facets that mean what a user expects — build the actual search query/facets sent
// to the backend, folding the synthetic "horror" pseudo-category into a keyword search.
function buildSearchParams() {
  const isHorror = filterCategories.value.includes('horror')
  const realCats = filterCategories.value.filter(c => c !== 'horror')
  const query = isHorror ? [searchInput.value.trim(), 'horror'].filter(Boolean).join(' ') : searchInput.value
  return { query, cats: realCats.length ? realCats : undefined }
}

function toggleCategory(name: string) {
  const idx = filterCategories.value.indexOf(name)
  if (idx === -1) filterCategories.value.push(name)
  else filterCategories.value.splice(idx, 1)
}

function removeCategory(name: string) {
  filterCategories.value = filterCategories.value.filter(c => c !== name)
  doSearch()
}

function clearCategories() {
  filterCategories.value = []
  doSearch()
}

async function loadVersions() {
  try {
    const manifest = await window.api.versions.listRemote()
    releaseVersions.value = manifest.versions
      .filter(v => v.type === 'release')
      .slice(0, 20)
      .map(v => v.id)
  } catch {}
}

async function loadCategories() {
  try {
    availableCategories.value = await window.api.modrinth.categories()
  } catch {}
}

// ── Search / results ──────────────────────────────────────────────────────────

const searchInput = ref('')
const results     = ref<ExploreHit[]>([])
const loading     = ref(false)
const error       = ref<string | null>(null)
const totalHits   = ref(0)
const offset      = ref(0)
const hasMore     = computed(() => results.value.length < totalHits.value)

let searchTimer: ReturnType<typeof setTimeout> | null = null

function triggerSearch() {
  if (searchTimer) clearTimeout(searchTimer)
  doSearch()
}

watch(searchInput, () => {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(doSearch, 350)
})

async function doSearch() {
  const type = activeType.value
  if (!type) return
  results.value   = []
  offset.value    = 0
  totalHits.value = 0
  error.value     = null
  loading.value   = true
  try {
    const { query, cats } = buildSearchParams()
    const res = await window.api.modrinth.exploreSearch(
      query,
      type,
      effectiveSource.value,
      filterVersion.value || undefined,
      filterLoader.value  || undefined,
      0,
      cats,
      filterSort.value,
    )
    results.value   = res.hits
    totalHits.value = res.total
    offset.value    = res.hits.length
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function loadMore() {
  const type = activeType.value
  if (!type || loading.value) return
  loading.value = true
  try {
    const { query, cats } = buildSearchParams()
    const res = await window.api.modrinth.exploreSearch(
      query,
      type,
      effectiveSource.value,
      filterVersion.value || undefined,
      filterLoader.value  || undefined,
      offset.value,
      cats,
      filterSort.value,
    )
    results.value.push(...res.hits)
    offset.value += res.hits.length
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function switchTab(key: string) {
  if (activeTab.value === key) return
  activeTab.value        = key
  searchInput.value      = ''
  filterLoader.value     = ''
  filterCategories.value = []
  results.value          = []
  error.value            = null
  if (key === 'servers') {
    refreshServers()
  } else {
    doSearch()
  }
}

// ── Install tracking ──────────────────────────────────────────────────────────

async function loadInstalls() {
  try {
    const data = await window.api.installs.get()
    // Populate installedMap from persisted records
    for (const [projectId, profileIds] of Object.entries(data.mods)) {
      installedMap.value.set(`modrinth-${projectId}`, new Set(profileIds))
    }
    installedMap.value = new Map(installedMap.value)
    // Populate serverAddedMap
    for (const [key, profileIds] of Object.entries(data.servers)) {
      serverAddedMap.value.set(key, new Set(profileIds))
    }
    serverAddedMap.value = new Map(serverAddedMap.value)
  } catch {}
}

// ── Profiles ──────────────────────────────────────────────────────────────────

const profiles        = ref<LaunchProfile[]>([])
const activeProfileId = ref<string | null>(null)

async function loadProfiles() {
  try {
    const [all, active] = await Promise.all([
      window.api.profiles.list(),
      window.api.profiles.getActive(),
    ])
    profiles.value        = all
    activeProfileId.value = active?.id ?? null
  } catch {}
}

// ── Mod install / picker ──────────────────────────────────────────────────────

const installingSet    = ref<Set<string>>(new Set())
const installedMap     = ref<Map<string, Set<string>>>(new Map())

const pickerHit        = ref<ExploreHit | null>(null)
const pickerSelected   = ref<string[]>([])
const pickerInstalling = ref(false)
const pickerPos        = ref<Record<string, string>>({})

function hitKey(hit: ExploreHit) { return `${hit.source}-${hit.id}` }

function openModPicker(hit: ExploreHit, event: MouseEvent) {
  const btn  = event.currentTarget as HTMLElement
  const rect = btn.getBoundingClientRect()
  const panelW = 280
  let left = rect.right - panelW
  if (left < 8) left = 8
  pickerPos.value = { top: `${rect.bottom + 6}px`, left: `${left}px` }
  pickerHit.value = hit
  const already = installedMap.value.get(hitKey(hit)) ?? new Set<string>()
  const def = activeProfileId.value
  pickerSelected.value = def && !already.has(def) ? [def] : []
}

function closePicker() {
  pickerHit.value      = null
  pickerSelected.value = []
}

// ── Modpack version picker — a modpack installs as its own new profile,
// so instead of asking "which existing profile(s)" it asks "which version". ──
const modpackPickerHit     = ref<ExploreHit | null>(null)
const modpackVersions      = ref<ModrinthVersion[]>([])
const modpackVerLoading    = ref(false)
const modpackInstalling    = ref(false)
const modpackVersionFilter = ref('')

// Distinct MC versions across all releases, newest-looking first (Modrinth
// already returns versions newest-published-first, so preserving that order
// here means the dropdown lists newer game versions before older ones too).
const modpackGameVersions = computed(() => {
  const seen = new Set<string>()
  const out: string[] = []
  for (const v of modpackVersions.value) {
    for (const gv of v.game_versions) {
      if (!seen.has(gv)) { seen.add(gv); out.push(gv) }
    }
  }
  return out
})

const filteredModpackVersions = computed(() => {
  if (!modpackVersionFilter.value) return modpackVersions.value
  return modpackVersions.value.filter(v => v.game_versions.includes(modpackVersionFilter.value))
})

async function openModpackPicker(hit: ExploreHit) {
  modpackPickerHit.value = hit
  modpackVersions.value  = []
  modpackVersionFilter.value = ''
  modpackVerLoading.value = true
  try {
    modpackVersions.value = await window.api.modrinth.versions(hit.id)
  } catch {
    modpackVersions.value = []
  } finally {
    modpackVerLoading.value = false
  }
}

function closeModpackPicker() {
  modpackPickerHit.value = null
  modpackVersions.value  = []
}

async function confirmModpackInstall(versionId: string) {
  const hit = modpackPickerHit.value
  if (!hit || modpackInstalling.value) return
  modpackInstalling.value = true
  try {
    const result = await window.api.modrinth.installModpack(hit.id, versionId)
    showToast({ title: 'Modpack installed', body: `"${result.name}" is ready as its own profile.`, variant: 'success', duration: 4000 })
    await loadProfiles()
  } catch (e) {
    showToast({ title: t('mods.toast.error', { msg: String(e) }), variant: 'error' })
  } finally {
    modpackInstalling.value = false
    closeModpackPicker()
  }
}

async function confirmModInstall() {
  const hit = pickerHit.value
  if (!hit || !pickerSelected.value.length || pickerInstalling.value) return

  const key = hitKey(hit)
  pickerInstalling.value = true
  installingSet.value = new Set([...installingSet.value, key])

  let anyError = false
  for (const profileId of pickerSelected.value) {
    try {
      await runModInstall(hit, profileId)
    } catch (e) {
      showToast({ title: t('mods.toast.error', { msg: String(e) }), variant: 'error' })
      anyError = true
    }
  }

  installingSet.value.delete(key)
  installingSet.value = new Set(installingSet.value)

  if (!anyError) {
    const existing = installedMap.value.get(key) ?? new Set<string>()
    pickerSelected.value.forEach(pid => existing.add(pid))
    installedMap.value = new Map(installedMap.value).set(key, existing)
    const n = pickerSelected.value.length
    showToast({ title: t('mods.toast.installed', { count: n }, n), variant: 'success' })
  }

  pickerInstalling.value = false
  closePicker()
}

const contentDragOver = ref(false)

async function onContentDrop(e: DragEvent) {
  contentDragOver.value = false
  const files = Array.from(e.dataTransfer?.files ?? []).filter(f => f.name.toLowerCase().endsWith('.jar'))
  if (!files.length) {
    showToast({ title: 'Drop a .jar file to install it', variant: 'info' })
    return
  }
  if (!activeProfileId.value) {
    showToast({ title: 'No active profile', body: 'Select a profile first, then drop mods to install them.', variant: 'warning' })
    return
  }
  let ok = 0
  for (const file of files) {
    const path = (file as File & { path?: string }).path
    if (!path) continue
    try {
      await window.api.mods.install(activeProfileId.value, path)
      ok++
    } catch (e) {
      showToast({ title: `Failed to install ${file.name}`, body: String(e), variant: 'error' })
    }
  }
  if (ok) showToast({ title: ok === 1 ? '1 mod installed' : `${ok} mods installed`, variant: 'success' })
}

// Modpacks never reach here — they route through openModpackPicker/confirmModpackInstall
// instead, since a modpack installs as its own new profile rather than into one.
async function runModInstall(hit: ExploreHit, profileId: string) {
  if (hit.source === 'curseforge') {
    await window.api.modrinth.installCurseforge(hit.id, hit.projectType, profileId)
    return
  }
  if (hit.projectType === 'mod')          await window.api.modrinth.installMod(hit.id, profileId)
  else if (hit.projectType === 'resourcepack') await window.api.modrinth.installResourcePack(hit.id, profileId)
  else if (hit.projectType === 'shader')  await window.api.modrinth.installShader(hit.id, profileId)
  else if (hit.projectType === 'datapack') await window.api.modrinth.installDatapack(hit.id, profileId)
}

// ── Servers ───────────────────────────────────────────────────────────────────

const servers       = ref<ServerStatus[]>([])
const serversLoading = ref(false)
const serversError  = ref<string | null>(null)

const showAddForm  = ref(false)
const newHost      = ref('')
const newPort      = ref(25565)
const newName      = ref('')
const addingServer = ref(false)

async function refreshServers() {
  serversLoading.value = true
  serversError.value   = null
  try {
    // Returns immediately with all servers in offline state
    servers.value = await window.api.servers.list()
  } catch (e) {
    serversError.value = String(e)
  } finally {
    serversLoading.value = false
  }
}

function applyPingResult(data: { id: string; online: boolean; favicon: string | null; version: string | null; playersOnline: number; playersMax: number; motd: string | null; ping: number }) {
  const idx = servers.value.findIndex(s => s.id === data.id)
  if (idx === -1) return
  servers.value[idx] = { ...servers.value[idx], ...data }
}

async function submitAddServer() {
  if (!newHost.value || addingServer.value) return
  const host = newHost.value.trim()
  const port = newPort.value || 25565
  const name = newName.value.trim() || host
  addingServer.value = true
  try {
    const id = await window.api.servers.add(host, port, name)
    showAddForm.value = false
    newHost.value = ''
    newPort.value = 25565
    newName.value = ''
    // Ping and add to list
    const status = await window.api.servers.ping(host, port)
    servers.value.push({
      id,
      name,
      host,
      port,
      featured:      false,
      online:        status !== null,
      favicon:       status?.favicon ?? null,
      version:       status?.version ?? null,
      playersOnline: status?.playersOnline ?? 0,
      playersMax:    status?.playersMax ?? 0,
      motd:          status?.motd ?? null,
      ping:          status?.ping ?? null,
    })
  } catch (e) {
    showToast({ title: t('mods.toast.error', { msg: String(e) }), variant: 'error' })
  } finally {
    addingServer.value = false
  }
}

async function deleteServer(id: string) {
  await window.api.servers.remove(id)
  servers.value = servers.value.filter(s => s.id !== id)
}

function pingClass(ping: number | null): string {
  if (ping === null) return ''
  if (ping < 100)  return 'ping-green'
  if (ping < 200)  return 'ping-yellow'
  return 'ping-red'
}

// Server profile picker
// key: "host:port" → Set<profileId>
const serverAddedMap        = ref<Map<string, Set<string>>>(new Map())
const serverPickerServer    = ref<ServerStatus | null>(null)
const serverPickerSelected  = ref<string[]>([])
const serverPickerInstalling = ref(false)

function serverKey(s: ServerStatus) { return `${s.host}:${s.port}` }

function openServerPicker(s: ServerStatus, event: MouseEvent) {
  const btn  = event.currentTarget as HTMLElement
  const rect = btn.getBoundingClientRect()
  const panelW = 280
  let left = rect.right - panelW
  if (left < 8) left = 8
  pickerPos.value = { top: `${rect.bottom + 6}px`, left: `${left}px` }
  serverPickerServer.value   = s
  const already = serverAddedMap.value.get(serverKey(s)) ?? new Set<string>()
  const def = activeProfileId.value
  serverPickerSelected.value = def && !already.has(def) ? [def] : []
}

function closeServerPicker() {
  serverPickerServer.value   = null
  serverPickerSelected.value = []
}

async function confirmServerAdd() {
  const s = serverPickerServer.value
  if (!s || !serverPickerSelected.value.length || serverPickerInstalling.value) return
  serverPickerInstalling.value = true
  let anyError = false
  const key = serverKey(s)
  for (const profileId of serverPickerSelected.value) {
    try {
      await window.api.servers.addToProfile(s.host, s.port, s.name, s.favicon, profileId)
      // Update in-memory map immediately
      const existing = serverAddedMap.value.get(key) ?? new Set<string>()
      existing.add(profileId)
      serverAddedMap.value = new Map(serverAddedMap.value).set(key, existing)
    } catch (e) {
      showToast({ title: t('mods.toast.error', { msg: String(e) }), variant: 'error' })
      anyError = true
    }
  }
  serverPickerInstalling.value = false
  if (!anyError) {
    const n = serverPickerSelected.value.length
    showToast({ title: t('mods.toast.serverAdded', { count: n }, n), variant: 'success' })
  }
  closeServerPicker()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function sourceUrl(hit: ExploreHit): string {
  return hit.source === 'modrinth'
    ? `https://modrinth.com/${hit.projectType}/${hit.slug}`
    : `https://www.curseforge.com/minecraft/mc-mods/${hit.slug}`
}

function openModMenu(event: MouseEvent, hit: ExploreHit): void {
  openContextMenu(event, [
    { label: 'View details', icon: 'eye', onClick: () => openDetails(hit) },
    { label: 'Open on source website', icon: 'external-link', onClick: () => window.open(sourceUrl(hit), '_blank') },
    { label: 'Copy project link', icon: 'link', onClick: () => {
      navigator.clipboard.writeText(sourceUrl(hit))
      showToast({ title: 'Link copied', variant: 'success', duration: 2500 })
    } },
  ])
}

// ── Mod details panel ────────────────────────────────────────────────────────

const detailsHit     = ref<ExploreHit | null>(null)
const detailsData    = ref<ModDetails | null>(null)
const detailsLoading = ref(false)
const detailsError   = ref<string | null>(null)

const detailsHtml = computed(() => {
  if (!detailsData.value?.description) return ''
  const raw = detailsData.value.descriptionFormat === 'markdown'
    ? (marked.parse(detailsData.value.description, { async: false }) as string)
    : detailsData.value.description
  return DOMPurify.sanitize(raw, { ADD_ATTR: ['target'] })
})

async function openDetails(hit: ExploreHit) {
  detailsHit.value     = hit
  detailsData.value    = null
  detailsError.value   = null
  detailsLoading.value = true
  try {
    detailsData.value = await window.api.modrinth.details(hit.id, hit.source)
  } catch (e) {
    detailsError.value = String(e)
  } finally {
    detailsLoading.value = false
  }
}

function closeDetails() {
  detailsHit.value  = null
  detailsData.value = null
}

function openSourceUrl(hit: ExploreHit) {
  window.open(sourceUrl(hit), '_blank')
}

function installFromDetails(hit: ExploreHit, event: MouseEvent) {
  if (hit.projectType === 'modpack') openModpackPicker(hit)
  else openModPicker(hit, event)
  closeDetails()
}

function formatNum(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000)     return `${(n / 1_000).toFixed(0)}K`
  return String(n)
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────

const listEl = ref<HTMLElement | null>(null)

onMounted(async () => {
  await Promise.all([loadProfiles(), loadVersions(), loadCategories(), loadInstalls()])
  window.api.modrinth.onProgress(msg => showToast({ title: msg, variant: 'info', duration: 3500 }))
  window.api.servers.onPingResult(data => applyPingResult(data as Parameters<typeof applyPingResult>[0]))
  doSearch()
})

onUnmounted(() => {
  if (searchTimer) clearTimeout(searchTimer)
})
</script>

<style lang="scss" scoped>
@font-face {
  src: url('../assets/fonts/mojangles.ttf') format('truetype');
  font-weight: normal;
  font-style: normal;
}

// ── Page shell ────────────────────────────────────────────────────────────────
.explore-page {
  height: 100%;
  display: grid;
  grid-template-columns: 1fr 0px;
  overflow: hidden;
  position: relative;
  // Smooth "push the explorer aside, reveal the panel" — animating grid-template-columns
  // (not just the panel's own width) is what makes the explorer itself visibly slide/shrink
  // rather than the panel just appearing on top of unchanged content.
  transition: grid-template-columns 420ms cubic-bezier(0.16, 1, 0.3, 1);

  &--split {
    grid-template-columns: 1fr 380px;
  }
}

.explore-main {
  display: flex;
  flex-direction: column;
  padding: 16px 20px;
  gap: 8px;
  overflow: hidden;
  min-width: 0;
  position: relative;
}

// ── Modpack version side panel ───────────────────────────────────────────────
.version-side-panel {
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
  background: #0d0d0d;
  border-left: 1px solid rgba(255, 255, 255, 0.1);
}

.version-side-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: thin;
  scrollbar-color: #282828 transparent;
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: #282828; }
}

.side-panel-enter-active,
.side-panel-leave-active {
  transition: opacity 260ms ease;
}
.side-panel-enter-from,
.side-panel-leave-to {
  opacity: 0;
}

// ── Search + tab pill row (Figma redesign, 2026-08-14 — pixel-sampled: flat
// #1A1A1E fill, no border, fully-rounded pill shape) ────────────────────────
.search-tab-row {
  position: relative;
  display: flex;
  gap: 10px;
  flex-shrink: 0;
  flex-wrap: wrap;
}

.tab-pill {
  height: 38px;
  padding: 0 22px;
  background: #1a1a1e;
  border: none;
  border-radius: 999px;
  color: rgba(255, 255, 255, 0.7);
  font-size: 13px;
  cursor: pointer;
  letter-spacing: 0.02em;
  transition: background 120ms, color 120ms;

  &:hover { background: #232328; color: rgba(255, 255, 255, 0.9); }
  &.active {
    background: #2a2a30;
    color: #fff;
  }
}

.search-bar {
  display: flex;
  align-items: center;
  background: #1a1a1e;
  border: none;
  border-radius: 999px;
  height: 38px;
  padding: 0 22px;
  flex: 1 1 320px;
  min-width: 200px;
}

.search-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.85);
  letter-spacing: 0.03em;
  &::placeholder { color: #666; }
}

// ── Controls row ──────────────────────────────────────────────────────────────
.controls-row {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
  align-items: center;
  flex-wrap: wrap;
}

.filter-select {
  height: 32px;
  background: none;
  border: none;
  color: #888;
  font-size: 10px;
  letter-spacing: 0.03em;
  padding: 0 8px;
  cursor: pointer;
  outline: none;
  appearance: none;
  border-radius: 0;
  transition: color 80ms;

  &:hover, &:focus { color: #bbb; }
  option { background: #111; color: #aaa; }
}

// ── Chip row ──────────────────────────────────────────────────────────────────
.chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  flex-shrink: 0;
}

.chip {
  font-size: 9px;
  color: #aaa;
  background: rgba(255,255,255,0.06);
  border: none;
  padding: 2px 8px;
  cursor: pointer;
  letter-spacing: 0.03em;
  transition: background 60ms, color 60ms;

  &:hover { background: rgba(255,255,255,0.1); color: #ccc; }
  &--clear { color: #555; background: transparent; &:hover { color: #888; } }
}

// ── Servers tab ───────────────────────────────────────────────────────────────
.servers-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 0;
  overflow: hidden;
}

.servers-topbar {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.server-action-btn {
  padding: 6px 16px;
  background: #0a0a0b;
  border: none;
  color: #888;
  font-size: 10px;
  letter-spacing: 0.04em;
  cursor: pointer;
  border-radius: 0;
  transition: background 80ms, color 80ms;
  display: flex;
  align-items: center;
  gap: 6px;

  &:hover:not(:disabled) { background: #111; color: #ccc; }
  &:disabled { opacity: 0.35; cursor: not-allowed; }
}

.add-server-form {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
  align-items: center;
  flex-wrap: wrap;
  padding: 10px 12px;
  background: rgba(10,10,11,0.85);
  border: 1px solid rgba(255,255,255,0.08);
}

.server-input {
  height: 30px;
  background: #0d0d0d;
  border: 1px solid rgba(118,119,120,0.5);
  color: #aaa;
  font-size: 10px;
  letter-spacing: 0.03em;
  padding: 0 8px;
  outline: none;
  border-radius: 0;
  flex: 1;
  min-width: 140px;

  &::placeholder { color: #444; }
  &:focus { border-color: rgba(200,200,200,0.5); color: #bbb; }
  &--port { flex: 0 0 70px; min-width: 70px; }
}

.server-add-confirm {
  height: 30px;
  padding: 0 16px;
  background: #111;
  border: none;
  color: #ccc;
  font-size: 10px;
  letter-spacing: 0.06em;
  cursor: pointer;
  border-radius: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  transition: background 80ms;

  &:hover:not(:disabled) { background: #1e1e1e; }
  &:disabled { opacity: 0.3; cursor: not-allowed; }
}

.server-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-height: 0;
  scrollbar-width: thin;
  scrollbar-color: #333 transparent;
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: #333; }
}

.server-card {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 18px 20px;
  background: #0f0f11;
  border: 1px solid #262627;
  border-radius: 16px;
  box-sizing: border-box;
  transition: background 80ms, border-color 80ms;
  flex-shrink: 0;

  &:hover { background: #141416; border-color: #333335; }
  &.offline { opacity: 0.5; }
}

.server-favicon-wrap {
  width: 72px;
  height: 72px;
  flex-shrink: 0;
  image-rendering: pixelated;
}

.server-favicon {
  width: 72px;
  height: 72px;
  object-fit: cover;
  image-rendering: pixelated;
  display: block;
}

.server-favicon-fallback {
  width: 72px;
  height: 72px;
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 30px;
  color: #444;
  text-transform: uppercase;
}

.server-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.server-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.server-name {
  font-size: 15px;
  color: #d9d9d9;
  letter-spacing: 0.02em;
}

.server-badge {
  font-size: 8px;
  color: #7aad7a;
  background: rgba(76,175,80,0.1);
  border: 1px solid rgba(76,175,80,0.3);
  padding: 1px 5px;
  letter-spacing: 0.06em;
}

.server-pinging-badge {
  font-size: 8px;
  color: #666;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.1);
  padding: 1px 5px;
  letter-spacing: 0.06em;
  animation: pulse 1.4s ease-in-out infinite;
}

@keyframes pulse { 0%, 100% { opacity: 0.4 } 50% { opacity: 1 } }

.server-offline-badge {
  font-size: 8px;
  color: #8b3333;
  background: rgba(139,51,51,0.1);
  border: 1px solid rgba(139,51,51,0.3);
  padding: 1px 5px;
  letter-spacing: 0.06em;
}

.server-motd {
  font-size: 11px;
  color: #666;
  letter-spacing: 0.02em;
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.server-ip {
  font-size: 10px;
  color: #444;
  letter-spacing: 0.03em;
}

.server-stats {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex-shrink: 0;
  min-width: 130px;
}

.stat-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.stat-label {
  font-size: 9px;
  color: #444;
  letter-spacing: 0.06em;
  min-width: 54px;
}

.stat-value {
  font-size: 11px;
  color: #888;
  letter-spacing: 0.02em;
}

.version-val {
  font-size: 8px;
  color: #555;
}

.ping-green  { color: #4caf50; }
.ping-yellow { color: #ffc107; }
.ping-red    { color: #f44336; }

.server-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 6px;
  flex-shrink: 0;
}

.server-remove-btn {
  background: none;
  border: none;
  color: #333;
  font-size: 11px;
  cursor: pointer;
  padding: 2px 4px;
  transition: color 80ms;
  &:hover { color: #888; }
}

// ── Content list ──────────────────────────────────────────────────────────────
.content-list {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  scrollbar-width: thin;
  scrollbar-color: #333 transparent;
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: #333; }

  &.drag-over { outline: 2px dashed $accent; outline-offset: -2px; }
}

.content-list--grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  align-content: start;
  gap: 28px;
  // The mod icon overlaps above each card's top edge — without this the
  // top row's icons get clipped by the scroll container's edge.
  padding-top: 22px;

  .load-more-row,
  .state-area {
    grid-column: 1 / -1;
  }
}

// ── View toggle ───────────────────────────────────────────────────────────────
.view-toggle {
  display: flex;
  flex-shrink: 0;
  background: #0a0a0b;
  border: 1px solid $border;
}

.view-toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  background: none;
  border: none;
  color: #555;
  cursor: pointer;
  transition: background 80ms, color 80ms;

  & + & { border-left: 1px solid $border; }
  &:hover { background: #1a1a1a; color: #aaa; }
  &.active { background: rgba(255,255,255,0.07); color: #d9d9d9; }
}

.drop-hint {
  flex-shrink: 0;
  padding: 10px 16px;
  margin-bottom: 6px;
  background: color-mix(in srgb, $accent 12%, transparent);
  border: 1px dashed $accent;
  border-radius: $radius-sm;
  color: $accent;
  font-size: 12px;
  font-weight: 600;
  text-align: center;
  pointer-events: none;
}

.state-area {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.skeleton-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.row-skeleton {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 16px 18px;
  background: rgba(10, 10, 11, 0.72);
  border: 1px solid $border;
}

.row-skeleton-lines {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.state-icon {
  color: #3a3a3a;
}

.state-text {
  font-size: 12px;
  color: #333;
  letter-spacing: 0.12em;
}

.error-text { color: #8b3333; }

.state-stack {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.retry-btn {
  padding: 6px 20px;
  font-size: 10px;
  color: #888;
  background: #0d0d0d;
  border: none;
  cursor: pointer;
  letter-spacing: 0.08em;
  transition: background 80ms, color 80ms, transform 100ms $ease-out;

  &:hover { background: #1a1a1a; color: #ccc; }
  &:active { transform: scale(0.96); }
}

// ── Mod row ───────────────────────────────────────────────────────────────────
// Same card look as the Hub's friends embed (.friends-card in HomePage.vue) —
// #0f0f11 fill, #262627 border, rounded — applied per-row here instead of once
// around the whole list, so each mod reads as its own embed.
.mod-row {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 16px 18px;
  background: #0f0f11;
  border: 1px solid #262627;
  border-radius: 16px;
  box-sizing: border-box;
  transition: background 80ms, border-color 80ms;
  flex-shrink: 0;

  &:hover { background: #141416; border-color: #333335; }
}

// Icon-grid layout: taller cards with the icon elevated over the top-right
// corner and an "ON" pill over the top-left when the mod is installed.
.content-list--grid .mod-row {
  position: relative;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
  padding: 20px 18px 18px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.05), transparent 90px), #1c1c1f;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 20px;
  overflow: visible;
  box-shadow: 0 10px 26px rgba(0, 0, 0, 0.35);

  &:hover { background: linear-gradient(180deg, rgba(255, 255, 255, 0.06), transparent 90px), #202023; border-color: rgba(255, 255, 255, 0.22); }
}

.content-list--grid .mod-on-pill {
  display: inline-flex;
  position: absolute;
  top: 14px;
  left: 14px;
}

.content-list--grid .mod-icon-wrap {
  position: absolute;
  top: -18px;
  right: 16px;
  width: 72px;
  height: 72px;
  border-radius: 14px;
  overflow: hidden;
  background: #1a1a1c;
  border: 1px solid rgba(255, 255, 255, 0.12);
  box-shadow: 0 10px 22px rgba(0, 0, 0, 0.5);
}

.content-list--grid .mod-icon,
.content-list--grid .mod-icon-fallback {
  width: 100%;
  height: 100%;
}

.content-list--grid .mod-info {
  width: 100%;
  margin-top: 46px;
}

.content-list--grid .mod-name-row {
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
}

.content-list--grid .mod-name {
  white-space: normal;
  font-size: 14px;
}

.content-list--grid .mod-tags {
  display: none;
}

.content-list--grid .mod-name-row .mod-stat {
  display: none;
}

.content-list--grid .install-area {
  width: 100%;
  justify-content: space-between;
}

.content-list--grid .mod-stat--footer {
  display: inline-flex;
  align-items: center;
}

.content-list--grid .install-btn {
  min-width: 0;
  padding: 8px 16px;
  font-size: 10px;
}

.mod-icon-wrap { width: 72px; height: 72px; flex-shrink: 0; }

.mod-icon {
  width: 72px;
  height: 72px;
  object-fit: cover;
  image-rendering: pixelated;
  display: block;
}

.mod-icon-fallback {
  width: 72px;
  height: 72px;
  background: #1a1a1a;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 26px;
  color: #555;
  text-transform: uppercase;
}

.mod-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.mod-name-row {
  display: flex;
  align-items: baseline;
  gap: 12px;
  flex-wrap: wrap;
}

.mod-name {
  font-size: 15px;
  color: #d9d9d9;
  letter-spacing: 0.03em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mod-name-btn {
  background: none;
  border: none;
  padding: 0;
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  max-width: 100%;
  transition: color 120ms $ease-out;

  &:hover { color: #fff; text-decoration: underline; }
}

.mod-stat {
  font-size: 10px;
  color: #555;
  letter-spacing: 0.02em;
  flex-shrink: 0;
}

.mod-stat--footer {
  display: none;
}

.mod-author {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 9px;
  color: #666;
  letter-spacing: 0.02em;
}

.mod-on-pill {
  display: none;
  align-items: center;
  gap: 6px;
  padding: 4px 10px 4px 8px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.16);
  color: #ccc;
  font-size: 8px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  flex-shrink: 0;
}

.mod-on-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #666;
}

.mod-on-pill--on {
  color: #d9d9d9;
}

.mod-on-pill--on .mod-on-dot {
  background: #4caf50;
  box-shadow: 0 0 6px rgba(76, 175, 80, 0.7);
}

.mod-desc {
  font-size: 10px;
  color: #666;
  letter-spacing: 0.02em;
  line-height: 1.6;
  margin: 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.mod-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.mod-tag {
  font-size: 9px;
  color: #555;
  background: rgba(255,255,255,0.04);
  border: 1px solid rgba(255,255,255,0.08);
  padding: 2px 7px;
  letter-spacing: 0.03em;
}

// ── Install area ──────────────────────────────────────────────────────────────
.install-area {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.details-btn {
  flex-shrink: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #777;
  cursor: pointer;
  transition: background 120ms $ease-out, color 120ms $ease-out, border-color 120ms $ease-out;

  &:hover { background: #1a1a1a; color: #ccc; border-color: rgba(255, 255, 255, 0.2); }
}

.install-btn {
  position: relative;
  flex-shrink: 0;
  padding: 10px 22px;
  font-size: 11px;
  color: #ccc;
  background: #111;
  border: 1px solid transparent;
  cursor: pointer;
  letter-spacing: 0.06em;
  overflow: hidden;
  transition: background 160ms $ease-out, border-color 160ms $ease-out, color 160ms $ease-out, transform 100ms $ease-out;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-width: 72px;

  &:hover:not(:disabled) { background: #1e1e1e; border-color: var(--accent, #{$primary}); color: #fff; }
  &:active:not(:disabled) { transform: scale(0.96); }
  &:disabled:has(.spinner) {
    opacity: 1;
    cursor: wait;
    border-color: color-mix(in srgb, var(--accent, #{$primary}) 50%, transparent);

    &::after {
      content: '';
      position: absolute;
      inset: 0;
      background: linear-gradient(90deg, transparent, color-mix(in srgb, var(--accent, #{$primary}) 30%, transparent), transparent);
      background-size: 60% 100%;
      animation: install-progress-sweep 1.1s ease-in-out infinite;
    }
  }
  &:disabled:not(:has(.spinner)) { opacity: 0.28; cursor: not-allowed; }
}

@keyframes install-progress-sweep {
  0%   { background-position: -100% 0; }
  100% { background-position: 200% 0; }
}

// ── Load more ─────────────────────────────────────────────────────────────────
.load-more-row {
  display: flex;
  justify-content: center;
  padding: 14px 0;
  flex-shrink: 0;
}

.load-more-btn {
  padding: 8px 32px;
  font-size: 10px;
  color: #888;
  background: #0d0d0d;
  border: none;
  cursor: pointer;
  letter-spacing: 0.08em;
  transition: background 80ms, color 80ms, transform 100ms $ease-out;
  display: flex;
  align-items: center;
  gap: 8px;

  &:hover:not(:disabled) { background: #1a1a1a; color: #ccc; }
  &:active:not(:disabled) { transform: scale(0.96); }
  &:disabled { opacity: 0.35; cursor: not-allowed; }
}

// ── Explore body (sidebar + content, side by side) ───────────────────────────
.explore-body {
  display: flex;
  gap: 16px;
  flex: 1;
  min-height: 0;
}

// Categories sidebar (pixel-sampled from Figma, 2026-08-14): flat #1A1A1E
// fill, no border, generously rounded, plain list — no chips/apply-button,
// filtering applies immediately on click.
.categories-sidebar {
  width: 220px;
  flex-shrink: 0;
  background: #1a1a1e;
  border-radius: 18px;
  padding: 18px 14px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow-y: auto;
  scrollbar-width: thin;
  scrollbar-color: #333 transparent;
}

.categories-title {
  font-size: 12px;
  font-weight: 700;
  color: #F6AE35;
  letter-spacing: 0.06em;
  font-family: 'Minecrafter', 'Mojangles', monospace;
  margin: 0 0 10px;
  padding: 0 10px;
}

.category-row {
  text-align: left;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.6);
  font-size: 12.5px;
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 100ms, color 100ms;

  &:hover { background: rgba(255, 255, 255, 0.05); color: rgba(255, 255, 255, 0.9); }
  &.active { background: rgba(255, 255, 255, 0.08); color: #fff; }
}

// ── Spinners ──────────────────────────────────────────────────────────────────
.spinner {
  border-radius: 50%;
  border-style: solid;
  border-top-color: #ccc;
  border-color: #444;
  animation: spin 0.7s linear infinite;
  flex-shrink: 0;

  &.sm { width: 11px; height: 11px; border-width: 1.5px; }
  &.lg { width: 24px; height: 24px; border-width: 2.5px; }
}

@keyframes spin { to { transform: rotate(360deg); } }

// ── Transitions ───────────────────────────────────────────────────────────────
.row-stagger-enter-active {
  transition: opacity 240ms ease, transform 240ms cubic-bezier(0.25, 1, 0.5, 1);
  transition-delay: calc(var(--i, 0) * 28ms);
}
.row-stagger-enter-from { opacity: 0; transform: translateY(8px); }
.row-stagger-leave-active { transition: opacity 120ms ease; }
.row-stagger-leave-to { opacity: 0; }
.row-stagger-move { transition: transform 220ms cubic-bezier(0.25, 1, 0.5, 1); }

.chip-pop-enter-active, .chip-pop-leave-active { transition: opacity 140ms ease, transform 140ms ease; }
.chip-pop-enter-from, .chip-pop-leave-to { opacity: 0; transform: scale(0.9); }
.chip-pop-move { transition: transform 160ms ease; }

.add-form-enter-active { transition: opacity 120ms, transform 120ms; }
.add-form-leave-active { transition: opacity 80ms; }
.add-form-enter-from   { opacity: 0; transform: translateY(-4px); }
.add-form-leave-to     { opacity: 0; }

// ── Mod details modal ────────────────────────────────────────────────────────
.details-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.72);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3500;
  backdrop-filter: blur(4px);
}

.details-modal {
  position: relative;
  width: 560px;
  max-width: 92vw;
  max-height: 82vh;
  display: flex;
  flex-direction: column;
  background: #121212;
  border: 1px solid $border;
  box-shadow: 0 24px 60px rgba(0, 0, 0, 0.55);
}

.details-close {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: #777;
  cursor: pointer;
  z-index: 1;
  transition: color 120ms $ease-out;

  &:hover { color: #fff; }
}

.details-header {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 20px 44px 16px 20px;
  border-bottom: 1px solid $border;
  flex-shrink: 0;
}

.details-icon-wrap {
  width: 48px;
  height: 48px;
  flex-shrink: 0;
}

.details-header-text {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.details-title {
  font-size: 16px;
  font-weight: 700;
  color: #fff;
  letter-spacing: 0.02em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.details-sub {
  font-size: 11px;
  color: #777;
  display: flex;
  align-items: center;
  gap: 6px;

  .sep { opacity: 0.5; }
}

.details-source-badge {
  font-size: 9px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: #999;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  padding: 2px 7px;
}

.details-body {
  flex: 1;
  overflow-y: auto;
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.details-stats-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 14px;
}

.details-stat {
  font-size: 10.5px;
  color: #888;
  letter-spacing: 0.02em;
}

.details-tags { margin: 0; }

.details-gallery {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  padding-bottom: 2px;
}

.details-gallery-img {
  height: 96px;
  width: auto;
  flex-shrink: 0;
  object-fit: cover;
  border: 1px solid $border;
}

.details-no-desc {
  -webkit-line-clamp: unset;
}

.details-description {
  font-size: 12px;
  line-height: 1.7;
  color: #bbb;

  :deep(h1), :deep(h2), :deep(h3), :deep(h4) {
    color: #eee;
    font-weight: 700;
    margin: 16px 0 8px;
    &:first-child { margin-top: 0; }
  }
  :deep(h1) { font-size: 16px; }
  :deep(h2) { font-size: 14.5px; }
  :deep(h3), :deep(h4) { font-size: 13px; }

  :deep(p) { margin: 0 0 10px; }
  :deep(a) { color: var(--accent, #{$primary}); text-decoration: underline; }
  :deep(ul), :deep(ol) { margin: 0 0 10px; padding-left: 20px; }
  :deep(li) { margin: 3px 0; }
  :deep(img) { max-width: 100%; height: auto; }
  :deep(code) {
    background: rgba(255, 255, 255, 0.08);
    padding: 1px 5px;
    font-size: 11px;
  }
  :deep(pre) {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid $border;
    padding: 10px;
    overflow-x: auto;
    code { background: none; padding: 0; }
  }
  :deep(blockquote) {
    margin: 0 0 10px;
    padding-left: 12px;
    border-left: 2px solid $border;
    color: #999;
  }
  :deep(hr) { border: none; border-top: 1px solid $border; margin: 14px 0; }
  :deep(table) { border-collapse: collapse; margin-bottom: 10px; }
  :deep(th), :deep(td) { border: 1px solid $border; padding: 4px 8px; font-size: 11px; }
}

.details-footer {
  display: flex;
  gap: 8px;
  padding: 14px 20px;
  border-top: 1px solid $border;
  flex-shrink: 0;
}

.modal-fade-enter-active, .modal-fade-leave-active { transition: opacity 160ms ease; }
.modal-fade-enter-from, .modal-fade-leave-to { opacity: 0; }
.modal-fade-enter-active .details-modal { animation: details-pop 180ms cubic-bezier(0.16, 1, 0.3, 1); }

@keyframes details-pop {
  from { opacity: 0; transform: scale(0.96) translateY(6px); }
  to   { opacity: 1; transform: scale(1) translateY(0); }
}
</style>

<style lang="scss">
@font-face {
  src: url('../assets/fonts/mojangles.ttf') format('truetype');
  font-weight: normal;
  font-style: normal;
}

.picker-overlay { position: fixed; inset: 0; z-index: 4000; }

.picker-panel {
  position: absolute;
  width: 280px;
  background: #0d0d0d;
  border: 1px solid rgba(255,255,255,0.22);
  box-shadow: 0 8px 32px rgba(0,0,0,0.85);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  max-height: 380px;
}

.picker-title {
  font-size: 11px;
  color: #d9d9d9;
  letter-spacing: 0.06em;
  margin: 0;
  padding: 12px 14px 4px;
}

// ── Version side panel header (icon + name, mirrors the mod row it came from) ──
.version-panel-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  flex-shrink: 0;
}

.version-panel-icon-wrap.mod-icon-wrap {
  width: 48px;
  height: 48px;

  .mod-icon, .mod-icon-fallback { width: 48px; height: 48px; font-size: 18px; }
}

.version-panel-header-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.version-panel-name {
  font-size: 13px;
  color: #f0f0f0;
  letter-spacing: 0.02em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.version-panel-sub {
  font-size: 9px;
  color: #666;
  letter-spacing: 0.05em;
}

.version-filter-row {
  padding: 10px 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.version-filter-select {
  width: 100%;
  height: 32px;
  background: #131313;
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #ccc;
  font-size: 10px;
  letter-spacing: 0.03em;
  padding: 0 10px;
  cursor: pointer;
  outline: none;
  border-radius: 0;
  transition: border-color 80ms;

  &:hover, &:focus { border-color: rgba(255, 255, 255, 0.3); }
}

.picker-sub {
  font-size: 9px;
  color: #555;
  letter-spacing: 0.03em;
  margin: 0;
  padding: 0 14px 10px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  border-bottom: 1px solid rgba(255,255,255,0.06);
}

.picker-empty {
  font-size: 10px;
  color: #444;
  padding: 14px;
  text-align: center;
  letter-spacing: 0.04em;
}

.picker-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 14px;
  cursor: pointer;
  transition: background 60ms;
  border-bottom: 1px solid rgba(255,255,255,0.04);
  overflow-y: auto;

  &:hover:not(.installed) { background: rgba(255,255,255,0.04); }
  &.checked    { background: rgba(255,255,255,0.05); }
  &.installed  { cursor: default; opacity: 0.55; }

  &--version {
    width: 100%;
    background: none;
    border: none;
    border-bottom: 1px solid rgba(255,255,255,0.04);
    text-align: left;
    font-family: inherit;

    &:hover:not(:disabled) { background: rgba(255,255,255,0.04); }
    &:disabled { cursor: not-allowed; opacity: 0.5; }
  }
}

.picker-check { width: 13px; height: 13px; accent-color: #fff; flex-shrink: 0; cursor: pointer; }

.picker-profile-info { display: flex; flex-direction: column; gap: 1px; min-width: 0; }

.picker-profile-name {
  font-size: 10px;
  color: #ccc;
  letter-spacing: 0.03em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.picker-profile-meta {
  font-size: 8px;
  color: #555;
  letter-spacing: 0.02em;
}

.picker-installed-label {
  font-size: 8px;
  color: #4caf50;
  letter-spacing: 0.04em;
  margin-left: auto;
  flex-shrink: 0;
}

.picker-footer {
  display: flex;
  gap: 6px;
  justify-content: flex-end;
  padding: 10px 14px;
  border-top: 1px solid rgba(255,255,255,0.06);
  flex-shrink: 0;
}

.picker-btn {
  padding: 6px 14px;
  font-size: 10px;
  cursor: pointer;
  letter-spacing: 0.04em;
  border: none;
  transition: background 80ms, color 80ms;
  display: flex;
  align-items: center;
  gap: 6px;
  border-radius: 0;

  &--cancel {
    background: transparent;
    color: #666;
    &:hover { color: #aaa; }
  }

  &--confirm {
    background: #111;
    color: #ccc;
    &:hover:not(:disabled) { background: #1e1e1e; color: #fff; }
    &:disabled { opacity: 0.28; cursor: not-allowed; }
  }
}

.picker-enter-active { transition: opacity 120ms, transform 120ms; }
.picker-leave-active { transition: opacity 80ms; }
.picker-enter-from   { opacity: 0; }
.picker-enter-from .picker-panel { transform: translateY(-4px); }
.picker-leave-to     { opacity: 0; }
</style>
