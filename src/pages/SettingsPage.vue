<template>
  <div class="settings-page">

    <!-- Content -->
    <div class="section-content">

      <div class="settings-toolbar">
        <div class="settings-search">
          <input v-model="settingsSearch" class="settings-search-input" placeholder="Search settings..." />
        </div>

        <button
          v-for="(name, i) in sectionNames"
          :key="name"
          class="settings-pill"
          :class="{ active: activeSection === i }"
          @click="jumpToSection(i)"
        >{{ name }}</button>

        <div class="advanced-switch">
          <span class="advanced-switch-label">Advanced</span>
          <div class="toggle" :class="{ 'toggle--on': s.launcher.advancedMode }" @click="s.launcher.advancedMode = !s.launcher.advancedMode; save()" />
        </div>
      </div>

      <div ref="sectionBodyRef" class="section-body">

      <!-- ── GAME ──────────────────────────────────────────────────────────── -->
      <div class="page-section">
        <h3 class="page-section-title">
          <component :is="IconGame" class="page-section-icon" />
          <span class="accent-green">Game</span>
        </h3>

        <span class="group-label">Paths</span>
        <div class="setting-group">
          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.game.dir') }}</span>
              <span class="setting-desc">{{ $t('settings.game.dirDesc') }}</span>
            </div>
            <div class="setting-control path-control">
              <span class="path-text">{{ s.game.defaultGameDir || $t('settings.game.dirDefault') }}</span>
              <button class="browse-btn" @click="pickGameDir">{{ $t('settings.browse') }}</button>
              <button v-if="s.game.defaultGameDir" class="clear-btn" @click="clearGameDir">✕</button>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 8h13v6a5 5 0 0 1-5 5H9a5 5 0 0 1-5-5V8z"/><path d="M17 9h1.5a2.5 2.5 0 0 1 0 5H17"/><path d="M8 2v2M12 2v2M16 2v2"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.game.java') }}</span>
              <span class="setting-desc">{{ $t('settings.game.javaDesc') }}</span>
            </div>
            <div class="setting-control path-control">
              <span class="path-text">{{ s.game.defaultJavaPath || $t('settings.game.javaAuto') }}</span>
              <button class="browse-btn" @click="pickJava">{{ $t('settings.browse') }}</button>
              <button v-if="s.game.defaultJavaPath" class="clear-btn" @click="clearJava">✕</button>
            </div>
          </div>
        </div>

        <span class="group-label">Memory</span>
        <div class="setting-group">
          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/><path d="M9 4V2M15 4V2M9 22v-2M15 22v-2M4 9H2M4 15H2M22 9h-2M22 15h-2"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.game.minRam') }}</span>
              <span class="setting-desc">{{ s.game.minRam }} MB</span>
            </div>
            <div class="setting-control slider-control">
              <span class="slider-val">{{ s.game.minRam }}M</span>
              <input
                type="range" class="slider" min="512" max="4096" step="256"
                :value="s.game.minRam"
                @input="s.game.minRam = +($event.target as HTMLInputElement).value; save()"
              />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="4" width="16" height="16" rx="2"/><path d="M9 4V2M15 4V2M9 22v-2M15 22v-2M4 9H2M4 15H2M22 9h-2M22 15h-2"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.game.maxRam') }}</span>
              <span class="setting-desc">{{ (s.game.maxRam / 1024).toFixed(1) }} GB</span>
            </div>
            <div class="setting-control slider-control">
              <span class="slider-val">{{ (s.game.maxRam / 1024).toFixed(1) }}G</span>
              <input
                type="range" class="slider" min="1024" max="16384" step="512"
                :value="s.game.maxRam"
                @input="s.game.maxRam = +($event.target as HTMLInputElement).value; save()"
              />
            </div>
          </div>
        </div>

        <span class="group-label">Window</span>
        <div class="setting-group">
          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="14" rx="1"/><path d="M8 20h8M12 18v2"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.game.resolution') }}</span>
              <span class="setting-desc">{{ $t('settings.game.resolutionDesc') }}</span>
            </div>
            <div class="setting-control res-control">
              <input
                type="number" class="num-input" placeholder="W"
                :value="s.game.resolution.width"
                @change="s.game.resolution.width = +($event.target as HTMLInputElement).value; save()"
              />
              <span class="res-x">×</span>
              <input
                type="number" class="num-input" placeholder="H"
                :value="s.game.resolution.height"
                @change="s.game.resolution.height = +($event.target as HTMLInputElement).value; save()"
              />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H5a2 2 0 0 0-2 2v3M16 3h3a2 2 0 0 1 2 2v3M8 21H5a2 2 0 0 1-2-2v-3M16 21h3a2 2 0 0 0 2-2v-3"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.game.fullscreen') }}</span>
              <span class="setting-desc">{{ $t('settings.game.fullscreenDesc') }}</span>
            </div>
            <div class="setting-control">
              <div class="toggle" :class="{ 'toggle--on': s.game.fullscreen }" @click="s.game.fullscreen = !s.game.fullscreen; save()" />
            </div>
          </div>

          <div class="setting-row setting-row--tall">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 9l3 3-3 3M12 15h5"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.game.jvmArgs') }}</span>
              <span class="setting-desc">{{ $t('settings.game.jvmArgsDesc') }}</span>
            </div>
            <div class="setting-control">
              <input
                type="text" class="text-input wide-input"
                placeholder="-XX:+UseG1GC -XX:MaxGCPauseMillis=50"
                :value="s.game.jvmArgs"
                @change="s.game.jvmArgs = ($event.target as HTMLInputElement).value; save()"
              />
            </div>
          </div>

          <div class="advanced-reveal" :class="{ open: s.launcher.advancedMode }">
            <div class="advanced-reveal-inner">
              <div class="setting-row setting-row--tall">
                <div class="setting-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 17l6-6-6-6"/><path d="M12 19h8"/></svg>
                </div>
                <div class="setting-info">
                  <span class="setting-label">Extra Game Arguments<span class="adv-badge">ADVANCED</span></span>
                  <span class="setting-desc">Custom flags appended to the Minecraft process, after everything else.</span>
                </div>
                <div class="setting-control">
                  <input
                    type="text" class="text-input wide-input"
                    placeholder="--quickPlaySingleplayer World1"
                    :value="s.game.extraGameArgs"
                    @change="s.game.extraGameArgs = ($event.target as HTMLInputElement).value; save()"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>

      </div>

      <!-- ── LAUNCHER ──────────────────────────────────────────────────────── -->
      <div class="page-section">
        <h3 class="page-section-title">
          <component :is="IconLauncher" class="page-section-icon" />
          Launcher
        </h3>

        <span class="group-label">Behavior</span>
        <div class="setting-group">
          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2v8"/><path d="M18.4 6.6a9 9 0 1 1-12.8 0"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.launcher.closeOnLaunch') }}</span>
              <span class="setting-desc">{{ $t('settings.launcher.closeOnLaunchDesc') }}</span>
            </div>
            <div class="setting-control">
              <div class="toggle" :class="{ 'toggle--on': s.launcher.closeOnLaunch }" @click="s.launcher.closeOnLaunch = !s.launcher.closeOnLaunch; save()" />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M21 2v6h-6"/><path d="M3 12a9 9 0 0 1 15-6.7L21 8"/><path d="M3 22v-6h6"/><path d="M21 12a9 9 0 0 1-15 6.7L3 16"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.launcher.keepOpen') }}</span>
              <span class="setting-desc">{{ $t('settings.launcher.keepOpenDesc') }}</span>
            </div>
            <div class="setting-control">
              <div class="toggle" :class="{ 'toggle--on': s.launcher.keepLauncherOpen }" @click="s.launcher.keepLauncherOpen = !s.launcher.keepLauncherOpen; save()" />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M8 17a5 5 0 0 1-1-9.9A6 6 0 0 1 18 9a4.5 4.5 0 0 1-1 8.9"/><path d="M12 11v6M9.5 14.5 12 17l2.5-2.5"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.launcher.autoUpdate') }}</span>
              <span class="setting-desc">{{ $t('settings.launcher.autoUpdateDesc') }}</span>
            </div>
            <div class="setting-control">
              <div class="toggle" :class="{ 'toggle--on': s.launcher.autoUpdate }" @click="s.launcher.autoUpdate = !s.launcher.autoUpdate; save()" />
            </div>
          </div>
        </div>

        <span class="group-label">Downloads</span>
        <div class="setting-group">
          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v12M7 10l5 5 5-5"/><path d="M5 21h14"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.launcher.concurrentDownloads') }}</span>
              <span class="setting-desc">{{ $t('settings.launcher.concurrentDownloadsDesc') }}</span>
            </div>
            <div class="setting-control slider-control">
              <span class="slider-val">{{ s.launcher.concurrentDownloads }}</span>
              <input
                type="range" class="slider" min="1" max="32" step="1"
                :value="s.launcher.concurrentDownloads"
                @input="s.launcher.concurrentDownloads = +($event.target as HTMLInputElement).value; save()"
              />
            </div>
          </div>
        </div>

        <span class="group-label">Sound</span>
        <div class="setting-group">
          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M5 9v6h4l5 4V5L9 9H5z"/><path d="M17 9a4 4 0 0 1 0 6"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.launcher.uiSounds') }}</span>
              <span class="setting-desc">{{ $t('settings.launcher.uiSoundsDesc') }}</span>
            </div>
            <div class="setting-control">
              <div class="toggle" :class="{ 'toggle--on': s.launcher.soundEnabled }" @click="s.launcher.soundEnabled = !s.launcher.soundEnabled; save()" />
            </div>
          </div>

          <div class="setting-row" :class="{ 'setting-row--muted': !s.launcher.soundEnabled }">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M5 9v6h4l5 4V5L9 9H5z"/><path d="M17 9a4 4 0 0 1 0 6M20 6a8 8 0 0 1 0 12"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.launcher.volume') }}</span>
              <span class="setting-desc">{{ s.launcher.soundVolume }}%</span>
            </div>
            <div class="setting-control slider-control">
              <span class="slider-val">{{ s.launcher.soundVolume }}%</span>
              <input
                type="range" class="slider" min="0" max="100" step="5"
                :value="s.launcher.soundVolume"
                :disabled="!s.launcher.soundEnabled"
                @input="s.launcher.soundVolume = +($event.target as HTMLInputElement).value; save()"
              />
            </div>
          </div>

          <div class="setting-row" :class="{ 'setting-row--muted': !s.launcher.soundEnabled }">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l10-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="16" cy="16" r="3"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.launcher.soundStyle') }}</span>
              <span class="setting-desc">{{ $t('settings.launcher.soundStyleDesc') }}</span>
            </div>
            <div class="setting-control">
              <div class="seg-control">
                <button
                  class="seg-btn"
                  :class="{ active: s.launcher.soundStyle === 'soft' }"
                  :disabled="!s.launcher.soundEnabled"
                  @click="s.launcher.soundStyle = 'soft'; save()"
                >{{ $t('settings.launcher.soft') }}</button>
                <button
                  class="seg-btn"
                  :class="{ active: s.launcher.soundStyle === 'clicky' }"
                  :disabled="!s.launcher.soundEnabled"
                  @click="s.launcher.soundStyle = 'clicky'; save()"
                >{{ $t('settings.launcher.clicky') }}</button>
              </div>
            </div>
          </div>
        </div>

        <span class="group-label">Controller</span>
        <div class="setting-group">
          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M6 8h12a4 4 0 0 1 4 4v1a3 3 0 0 1-5.4 1.8L15 13H9l-1.6 1.8A3 3 0 0 1 2 13v-1a4 4 0 0 1 4-4z"/><path d="M8 10.5v2M7 11.5h2"/><circle cx="16" cy="11" r="0.6" fill="currentColor"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">Haptic Feedback</span>
              <span class="setting-desc">Rumble your controller when you move, select and go back in the launcher.</span>
            </div>
            <div class="setting-control">
              <div class="toggle" :class="{ 'toggle--on': s.launcher.controllerHaptics }" @click="s.launcher.controllerHaptics = !s.launcher.controllerHaptics; save(); if (s.launcher.controllerHaptics) rumble('confirm')" />
            </div>
          </div>

          <div class="setting-row" :class="{ 'setting-row--muted': !s.launcher.controllerHaptics }">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12h2l2-5 3 10 3-8 2 3h6"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">Strength</span>
              <span class="setting-desc">{{ s.launcher.hapticsStrength }}%</span>
            </div>
            <div class="setting-control slider-control">
              <span class="slider-val">{{ s.launcher.hapticsStrength }}%</span>
              <input
                type="range" class="slider" min="10" max="100" step="10"
                :value="s.launcher.hapticsStrength"
                :disabled="!s.launcher.controllerHaptics"
                @input="s.launcher.hapticsStrength = +($event.target as HTMLInputElement).value; save()"
                @change="rumble('confirm')"
              />
            </div>
          </div>

          <div class="setting-row" :class="{ 'setting-row--muted': !s.launcher.controllerHaptics }">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M13 2L4 14h7l-1 8 9-12h-7l1-8z"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">Test</span>
              <span class="setting-desc">{{ padPlugged ? 'Feel what a click, a back and a bump are like.' : 'Connect a controller and press a button on it first.' }}</span>
            </div>
            <div class="setting-control">
              <div class="seg-control">
                <button class="seg-btn" :disabled="!s.launcher.controllerHaptics" @click="rumble('move')">Move</button>
                <button class="seg-btn" :disabled="!s.launcher.controllerHaptics" @click="rumble('confirm')">Select</button>
                <button class="seg-btn" :disabled="!s.launcher.controllerHaptics" @click="rumble('bump')">Bump</button>
              </div>
            </div>
          </div>
        </div>

        <span class="group-label">Integrations</span>
        <div class="setting-group">
          <div class="setting-row setting-row--tall">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="7.5" cy="15.5" r="4.5"/><path d="M10.5 12.5 20 3M17 6l3 3M14 9l2 2"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.launcher.curseforgeKey') }}</span>
              <span class="setting-desc">{{ $t('settings.launcher.curseforgeKeyDesc') }}</span>
            </div>
            <div class="setting-control">
              <input
                type="password" class="text-input wide-input"
                placeholder="••••••••••••••••••••"
                :value="s.launcher.curseforgeApiKey"
                @change="s.launcher.curseforgeApiKey = ($event.target as HTMLInputElement).value; save()"
              />
            </div>
          </div>
        </div>

        <div class="advanced-reveal" :class="{ open: s.launcher.advancedMode }">
          <div class="advanced-reveal-inner">
            <span class="group-label">Advanced</span>
            <div class="setting-group">
              <div class="setting-row">
                <div class="setting-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M7 9l3 3-3 3M12 15h5"/></svg>
                </div>
                <div class="setting-info">
                  <span class="setting-label">Debug Logging<span class="adv-badge">ADVANCED</span></span>
                  <span class="setting-desc">Log the full Java command line and library count on every launch.</span>
                </div>
                <div class="setting-control">
                  <div class="toggle" :class="{ 'toggle--on': s.launcher.debugLogging }" @click="s.launcher.debugLogging = !s.launcher.debugLogging; save()" />
                </div>
              </div>
            </div>
          </div>
        </div>

      </div>

      <!-- ── APPEARANCE ────────────────────────────────────────────────────── -->
      <div class="page-section">
        <h3 class="page-section-title">
          <component :is="IconAppearance" class="page-section-icon" />
          Appearance
        </h3>

        <span class="group-label">Theme</span>
        <div class="setting-group">
          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="14" rx="1"/><path d="M3 9h18"/><circle cx="6.5" cy="6.5" r="0.6" fill="currentColor" stroke="none"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">Style</span>
              <span class="setting-desc">Swaps the launcher's entire look. Windows 95 is a fun retro skin - give it a spin.</span>
            </div>
            <div class="setting-control">
              <div class="seg-control">
                <button class="seg-btn" :class="{ active: s.appearance.theme === 'default' }" @click="s.appearance.theme = 'default'; save()">Default</button>
                <button class="seg-btn" :class="{ active: s.appearance.theme === 'win95' }" @click="s.appearance.theme = 'win95'; save()">Windows 95</button>
              </div>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M9.06 11.9l8.07-8.06a2.85 2.85 0 1 1 4.03 4.03l-8.06 8.08"/><path d="M7.07 14.94c-1.66 0-3 1.35-3 3.02 0 1.33-2.5 1.52-2 2.02 1.08 1.1 2.49 2.02 4 2.02 2.2 0 4-1.8 4-4.04a3.01 3.01 0 0 0-3-3.02z"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.appearance.accentColor') }}</span>
              <span class="setting-desc">{{ $t('settings.appearance.accentColorDesc') }}</span>
            </div>
            <div class="setting-control accent-control">
              <div class="color-swatches">
                <button
                  v-for="c in accentPresets"
                  :key="c"
                  class="color-swatch"
                  :class="{ active: s.appearance.accentColor === c }"
                  :style="{ background: c }"
                  @click="s.appearance.accentColor = c; save()"
                />
              </div>
              <label class="color-custom" title="Custom color">
                <input
                  type="color"
                  class="color-input"
                  :value="s.appearance.accentColor"
                  @input="s.appearance.accentColor = ($event.target as HTMLInputElement).value; save()"
                />
                <span class="color-hex">{{ s.appearance.accentColor }}</span>
              </label>
            </div>
          </div>
        </div>

        <span class="group-label">Language</span>
        <div class="setting-group">
          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15 15 0 0 1 0 20 15 15 0 0 1 0-20z"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">{{ $t('settings.appearance.language') }}</span>
              <span class="setting-desc">{{ $t('settings.appearance.languageDesc') }}</span>
            </div>
            <div class="setting-control">
              <select
                class="select-input"
                :value="s.appearance.language"
                @change="s.appearance.language = ($event.target as HTMLSelectElement).value; save()"
              >
                <option v-for="code in availableLocales" :key="code" :value="code">{{ localeDisplayName(code) }}</option>
              </select>
            </div>
          </div>
        </div>

      </div>

      <!-- ── ANIMATIONS ───────────────────────────────────────────────────── -->
      <div class="page-section">
        <h3 class="page-section-title">
          <component :is="IconAnimations" class="page-section-icon" />
          Animations
        </h3>

        <span class="group-label">Motion</span>
        <div class="setting-group">
          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 12h4l3-8 4 16 3-8h4"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">Reduce Motion</span>
              <span class="setting-desc">Collapses animations and transitions app-wide to near-instant.</span>
            </div>
            <div class="setting-control">
              <div class="seg-control">
                <button class="seg-btn" :class="{ active: s.appearance.reduceMotion === 'system' }" @click="s.appearance.reduceMotion = 'system'; save()">System</button>
                <button class="seg-btn" :class="{ active: s.appearance.reduceMotion === 'off' }" @click="s.appearance.reduceMotion = 'off'; save()">Full</button>
                <button class="seg-btn" :class="{ active: s.appearance.reduceMotion === 'on' }" @click="s.appearance.reduceMotion = 'on'; save()">Reduced</button>
              </div>
            </div>
          </div>
        </div>

        <span class="group-label">Interface</span>
        <div class="setting-group">
          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M8 12l3 3 5-6"/><circle cx="12" cy="12" r="9"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">Hover Effects</span>
              <span class="setting-desc">Lift/scale micro-interactions when hovering buttons and cards.</span>
            </div>
            <div class="setting-control">
              <div class="toggle" :class="{ 'toggle--on': !s.appearance.disableHoverEffects }" @click="s.appearance.disableHoverEffects = !s.appearance.disableHoverEffects; save()" />
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5z"/><path d="M2 17l10 5 10-5"/><path d="M2 12l10 5 10-5"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">Splash Screen</span>
              <span class="setting-desc">Show the BejaClient intro screen while the launcher loads.</span>
            </div>
            <div class="setting-control">
              <div class="toggle" :class="{ 'toggle--on': !s.appearance.disableSplashScreen }" @click="s.appearance.disableSplashScreen = !s.appearance.disableSplashScreen; save()" />
            </div>
          </div>
        </div>

      </div>

      <!-- ── VOICE ────────────────────────────────────────────────────────── -->
      <div class="page-section">
        <h3 class="page-section-title">
          <component :is="IconVoice" class="page-section-icon" />
          Voice
        </h3>

        <span class="group-label">Lobby voice chat</span>
        <div class="setting-group">
          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="2" width="6" height="12" rx="3"/><path d="M5 11a7 7 0 0 0 14 0"/><line x1="12" y1="18" x2="12" y2="22"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">Microphone</span>
              <span class="setting-desc">{{ micError || 'The input device used in party voice chat. Applies right away, also during a call.' }}</span>
            </div>
            <div class="setting-control">
              <select class="select-input mic-select" :value="micId" @change="onMicChange(($event.target as HTMLSelectElement).value)">
                <option value="">Default microphone</option>
                <option v-for="d in micDevices" :key="d.deviceId" :value="d.deviceId">{{ d.label || `Microphone ${d.deviceId.slice(0, 6)}` }}</option>
              </select>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 12h2M8 8v8M12 5v14M16 8v8M20 12h-2"/></svg>
            </div>
            <div class="setting-info">
              <span class="setting-label">Input level</span>
              <span class="setting-desc">Speak to check that this is the microphone you want.</span>
            </div>
            <div class="setting-control">
              <div class="mic-meter"><div class="mic-meter-fill" :style="{ width: micLevel + '%' }" /></div>
            </div>
          </div>
        </div>
      </div>

      <!-- ── ACCOUNTS ─────────────────────────────────────────────────────── -->
      <div class="page-section">
        <h3 class="page-section-title">
          <component :is="IconAccounts" class="page-section-icon" />
          Accounts
        </h3>
        <AccountsSettings />
      </div>

      <!-- ── ABOUT ────────────────────────────────────────────────────────── -->
      <div class="page-section">
        <h3 class="page-section-title">
          <component :is="IconAbout" class="page-section-icon" />
          About
        </h3>
        <div class="setting-group">
          <div class="setting-row about-links">
            <button class="about-link" @click="openExternal('https://bejaclient.com/terms')">{{ $t('nav.footer.terms') }}</button>
            <button class="about-link" @click="openExternal('https://bejaclient.com/privacy')">{{ $t('nav.footer.privacy') }}</button>
            <button class="about-link" @click="openExternal('https://bejaclient.com/support')">{{ $t('nav.footer.support') }}</button>
          </div>
        </div>
      </div>

      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h, watch, nextTick, onMounted, onActivated, onDeactivated, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../store/settingsStore'
import AccountsSettings from './settings/AccountsSettings.vue'
import { getVoiceInputId, setVoiceInputId } from '../composables/voiceDevice'
import { rumble, padConnected as padPlugged } from '../composables/useGamepad'

const settingsStore = useSettingsStore()
const s = computed(() => settingsStore.settings)

// ── Language options ──────────────────────────────────────────────────────────
// Derived from the locales actually registered in i18n/index.ts (vue-i18n exposes
// them via availableLocales) instead of a hand-maintained list that can silently
// drift out of sync with which locale files really exist.
const { availableLocales } = useI18n()

function localeDisplayName(code: string): string {
  try {
    return new Intl.DisplayNames([code], { type: 'language' }).of(code) ?? code
  } catch {
    return code
  }
}

async function save() { await settingsStore.save() }
async function pickGameDir() { await settingsStore.chooseGameDir() }
async function pickJava()    { await settingsStore.chooseJava() }
function clearGameDir() { s.value.game.defaultGameDir = ''; save() }
function clearJava()    { s.value.game.defaultJavaPath = ''; save() }

// ── Voice: microphone choice + live level ──────────────────────────────────────
const micDevices = ref<MediaDeviceInfo[]>([])
const micId      = ref(getVoiceInputId())
const micLevel   = ref(0)
const micError   = ref('')
let micStream: MediaStream | null = null
let micCtx: AudioContext | null = null
let micFrame: number | null = null

async function listMics(): Promise<void> {
  try {
    micDevices.value = (await navigator.mediaDevices.enumerateDevices()).filter(d => d.kind === 'audioinput' && d.deviceId !== 'default' && d.deviceId !== 'communications')
  } catch { micDevices.value = [] }
}

function stopMicMeter(): void {
  if (micFrame !== null) cancelAnimationFrame(micFrame)
  micFrame = null
  micStream?.getTracks().forEach(t => t.stop())
  micStream = null
  micCtx?.close().catch(() => {})
  micCtx = null
  micLevel.value = 0
}

async function startMicMeter(): Promise<void> {
  stopMicMeter()
  try {
    micStream = await navigator.mediaDevices.getUserMedia({
      audio: micId.value ? { deviceId: { exact: micId.value } } : true,
      video: false,
    })
    micError.value = ''
  } catch (e) {
    const name = (e as { name?: string })?.name
    if (micId.value && name === 'OverconstrainedError') { micId.value = ''; setVoiceInputId(''); return startMicMeter() }
    micError.value = name === 'NotAllowedError' ? 'Microphone access denied. Allow it in your system settings.' : 'No microphone found.'
    await listMics()
    return
  }
  await listMics()   // device names are only filled in once permission was granted
  micCtx = new AudioContext()
  const analyser = micCtx.createAnalyser()
  analyser.fftSize = 512
  micCtx.createMediaStreamSource(micStream).connect(analyser)
  const buf = new Uint8Array(analyser.frequencyBinCount)
  const tick = () => {
    micFrame = requestAnimationFrame(tick)
    analyser.getByteFrequencyData(buf)
    const avg = buf.reduce((a, v) => a + v, 0) / buf.length
    micLevel.value = Math.min(100, Math.round((avg / 128) * 250))
  }
  tick()
}

function onMicChange(id: string): void {
  micId.value = id
  setVoiceInputId(id)
  void startMicMeter()
}

// The page is kept alive between visits: only hold the microphone open while it is on screen.
onActivated(() => {
  void startMicMeter()
  navigator.mediaDevices?.addEventListener?.('devicechange', listMics)
})
onDeactivated(() => {
  stopMicMeter()
  navigator.mediaDevices?.removeEventListener?.('devicechange', listMics)
})
onUnmounted(stopMicMeter)

const accentPresets = ['#27ade0', '#f97316', '#30d158', '#e879f9', '#fbbf24', '#f87171', '#60a5fa', '#a78bfa']

// ── Section icons (inline SVG components) ─────────────────────────────────────
const IconVoice = {
  render: () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.8, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }, [
    h('rect', { x: 9, y: 2, width: 6, height: 12, rx: 3 }),
    h('path', { d: 'M5 11a7 7 0 0 0 14 0' }),
    h('line', { x1: 12, y1: 18, x2: 12, y2: 22 }),
  ]),
}
const IconGame = {
  render: () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.8, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }, [
    h('rect', { x: 2, y: 3, width: 20, height: 14, rx: 2 }),
    h('line', { x1: 8, y1: 21, x2: 16, y2: 21 }),
    h('line', { x1: 12, y1: 17, x2: 12, y2: 21 }),
  ])
}

const IconLauncher = {
  render: () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.8, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }, [
    h('path', { d: 'M12 2L2 7l10 5 10-5-10-5z' }),
    h('path', { d: 'M2 17l10 5 10-5' }),
    h('path', { d: 'M2 12l10 5 10-5' }),
  ])
}

const IconAppearance = {
  render: () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.8, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }, [
    h('circle', { cx: 12, cy: 12, r: 10 }),
    h('path', { d: 'M12 2a14.5 14.5 0 0 0 0 20 10 10 0 0 0 0-20' }),
  ])
}

const IconAccounts = {
  render: () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.8, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }, [
    h('circle', { cx: 12, cy: 8, r: 4 }),
    h('path', { d: 'M4 20c0-4 3.6-7 8-7s8 3 8 7' }),
  ])
}

const IconAnimations = {
  render: () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.8, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }, [
    h('path', { d: 'M4 12h4l3-8 4 16 3-8h4' }),
  ])
}

const IconAbout = {
  render: () => h('svg', { viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': 1.8, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' }, [
    h('circle', { cx: 12, cy: 12, r: 10 }),
    h('line', { x1: 12, y1: 16, x2: 12, y2: 11 }),
    h('line', { x1: 12, y1: 8, x2: 12.01, y2: 8 }),
  ])
}

function openExternal(url: string) {
  window.open(url, '_blank')
}

// ── Settings search ────────────────────────────────────────────────────────────
// Rows are hand-authored template blocks, not a data model, so filtering works
// directly against the rendered DOM text rather than requiring a full
// data-driven rewrite of every section.
const settingsSearch = ref('')
const sectionBodyRef = ref<HTMLElement | null>(null)

function applySettingsFilter() {
  const q = settingsSearch.value.trim().toLowerCase()
  const rows = sectionBodyRef.value?.querySelectorAll<HTMLElement>('.setting-row')
  rows?.forEach(row => {
    const match = !q || (row.textContent ?? '').toLowerCase().includes(q)
    row.classList.toggle('setting-row--hidden', !match)
  })
}

watch(settingsSearch, () => nextTick(applySettingsFilter))

// ── Section pills: jump to a section of the long page ─────────────────────────
const sectionNames = ref<string[]>([])
const activeSection = ref(0)

function sectionEls(): HTMLElement[] {
  return Array.from(sectionBodyRef.value?.querySelectorAll<HTMLElement>('.page-section') ?? [])
}

function jumpToSection(i: number): void {
  activeSection.value = i
  sectionEls()[i]?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

onMounted(() => {
  sectionNames.value = sectionEls().map(el => (el.querySelector('.page-section-title')?.textContent ?? '').trim()).filter(Boolean)
})
</script>

<style lang="scss" scoped>
// ── Page shell ────────────────────────────────────────────────────────────────
.settings-page {
  height: 100%;
  display: flex;
  overflow: hidden;
  position: relative;
}

// ── Section content ───────────────────────────────────────────────────────────
.section-content {
  flex: 1;
  min-width: 0;
  overflow-y: auto;
  padding: 20px 28px 36px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  scrollbar-width: thin;
  scrollbar-color: #282828 transparent;
  &::-webkit-scrollbar { width: 4px; }
  &::-webkit-scrollbar-thumb { background: #282828; }
}

// Header in the Explore look: search bar, section pills, advanced switch on one row.
.settings-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
  flex-wrap: wrap;
}

.settings-search {
  flex: 1;
  min-width: 200px;
}

.settings-search-input {
  width: 100%;
  height: 38px;
  background: #13141A;
  border: none;
  border-radius: $radius-sm;
  color: rgba(255, 255, 255, 0.85);
  font-size: 12px;
  letter-spacing: 0.03em;
  padding: 0 14px;
  outline: none;

  &::placeholder { color: #666; }
}

.settings-pill {
  height: 38px;
  padding: 0 20px;
  background: #13141A;
  border: none;
  border-radius: 12px;
  color: rgba(255, 255, 255, 0.7);
  font-size: 13px;
  cursor: pointer;
  letter-spacing: 0.02em;
  white-space: nowrap;
  transition: background 120ms, color 120ms;

  &:hover { background: #232328; color: rgba(255, 255, 255, 0.9); }
  &.active { background: #2a2a30; color: #fff; }
}

.advanced-switch {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
  height: 38px;
  padding: 0 14px;
  background: #13141A;
  border-radius: 12px;
}

.advanced-switch-label {
  font-size: 13px;
  letter-spacing: 0.02em;
  color: rgba(255, 255, 255, 0.7);
}

.section-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

// ── Page section (Game / Launcher / Appearance / Accounts) ───────────────────
.page-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-top: 26px;

  &:first-child { padding-top: 6px; }

  // Collapse a whole page section once search hides every row inside it.
  &:not(:has(.setting-row:not(.setting-row--hidden))) {
    display: none;
  }
}

.page-section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: 'Minecrafter', 'Mojangles', monospace;
  font-weight: 700;
  font-size: 14px;
  color: #F6AE35;
  letter-spacing: 0.06em;
  margin: 0;

  .accent-green { color: inherit; }
}

.page-section-icon {
  width: 16px;
  height: 16px;
  color: rgba(246, 174, 53, 0.6);
}

// ── Group label ───────────────────────────────────────────────────────────────
.group-label {
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.4);
  letter-spacing: 0.06em;
  margin: 10px 2px -2px;

  &:first-child { margin-top: 0; }
}

// ── Advanced-mode reveal ─────────────────────────────────────────────────────
// Grid-rows trick animates height without measuring - the row track goes
// 0fr → 1fr while the inner wrapper clips content during the transition.
.advanced-reveal {
  display: grid;
  grid-template-rows: 0fr;
  margin-top: -8px;
  opacity: 0;
  transition: grid-template-rows 380ms cubic-bezier(0.16, 1, 0.3, 1), margin-top 380ms cubic-bezier(0.16, 1, 0.3, 1), opacity 260ms ease;

  &.open {
    grid-template-rows: 1fr;
    margin-top: 0;
    opacity: 1;
    transition: grid-template-rows 380ms cubic-bezier(0.16, 1, 0.3, 1), margin-top 380ms cubic-bezier(0.16, 1, 0.3, 1), opacity 320ms ease 100ms;
  }
}

.advanced-reveal-inner {
  overflow: hidden;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.adv-badge {
  display: inline-block;
  font-family: $font-family;
  font-weight: 700;
  font-size: 11px;
  letter-spacing: 0.08em;
  color: #fff;
  background: #cc0000;
  padding: 2px 5px;
  margin-left: 6px;
  vertical-align: middle;
}

// ── Setting group ─────────────────────────────────────────────────────────────
.setting-group {
  display: flex;
  flex-direction: column;
  gap: 8px;

  // Collapse a group entirely once search has hidden every row inside it.
  &:not(:has(.setting-row:not(.setting-row--hidden))) {
    display: none;
  }
}

// ── Setting row ───────────────────────────────────────────────────────────────
.setting-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 18px;
  background: #13141A;
  border: none;
  border-radius: $radius-sm;
  min-height: 60px;
  transition: background 80ms;

  &:hover { background: #181920; }
  &--tall { align-items: flex-start; padding-top: 16px; padding-bottom: 16px; }
  &--tall .setting-icon { margin-top: 1px; }
  &--muted { opacity: 0.4; pointer-events: none; }
  &--hidden { display: none; }
}

.about-links {
  gap: 22px;
}

.about-link {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  font-family: $font-family;
  font-size: 18px;
  font-weight: 500;
  letter-spacing: 0.01em;
  color: rgba(255, 255, 255, 0.5);
  transition: color 160ms $ease-out;

  &:hover { color: rgba(255, 255, 255, 0.9); }
}

// ── Row leading icon ──────────────────────────────────────────────────────────
.setting-icon {
  width: 34px;
  height: 34px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #0f1014;
  border: none;
  border-radius: $radius-sm;
  color: rgba(255, 255, 255, 0.55);

  svg { width: 16px; height: 16px; }
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
  flex: 1;
  max-width: 55%;
}

.setting-label {
  font-weight: 500;
  font-size: 14px;
  color: #fff;
  letter-spacing: 0.02em;
}

.setting-desc {
  font-weight: 500;
  font-size: 11.5px;
  color: rgba(255, 255, 255, 0.45);
  letter-spacing: 0.03em;
  line-height: 1.5;
}

// ── Setting control wrapper ───────────────────────────────────────────────────
.setting-control {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  margin-left: auto;
}

// ── Toggle switch ─────────────────────────────────────────────────────────────
.toggle {
  width: 40px;
  height: 22px;
  background: #232328;
  border: none;
  box-shadow: none;
  border-radius: 12px;
  position: relative;
  cursor: pointer;
  transition: background 150ms;
  flex-shrink: 0;

  &::after {
    content: '';
    position: absolute;
    width: 16px;
    height: 16px;
    background: rgba(255, 255, 255, 0.55);
    border-radius: 50%;
    top: 3px;
    left: 3px;
    transition: transform 150ms cubic-bezier(0.2,0,0,1), background 150ms;
  }

  &--on {
    background: var(--accent, #27ade0);
    box-shadow: none;

    &::after {
      background: #fff;
      transform: translateX(18px);
    }
  }
}

// ── Slider ────────────────────────────────────────────────────────────────────
.slider-control {
  gap: 10px;
}

.slider-val {
  font-weight: 500;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.6);
  letter-spacing: 0.04em;
  min-width: 36px;
  text-align: right;
  flex-shrink: 0;
}

.slider {
  width: 140px;
  height: 3px;
  -webkit-appearance: none;
  background: #2a2a2a;
  border-radius: $radius-sm;
  outline: none;
  cursor: pointer;

  &::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 13px;
    height: 13px;
    background: #fff;
    border-radius: 50%;
    cursor: pointer;
    transition: transform 100ms;
    &:hover { transform: scale(1.2); }
  }

  &:disabled {
    opacity: 0.3;
    cursor: not-allowed;
    &::-webkit-slider-thumb { cursor: not-allowed; }
  }
}

// ── Text input ────────────────────────────────────────────────────────────────
.text-input {
  height: 34px;
  background: #0f1014;
  border: none;
  border-radius: $radius-sm;
  color: #fff;
  font-weight: 500;
  font-size: 13px;
  letter-spacing: 0.02em;
  padding: 0 12px;
  outline: none;
  transition: box-shadow 100ms;
  width: 200px;

  &:focus { box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.3); }
  &::placeholder { color: #555; }

  &.wide-input { width: 260px; }
}

// ── Number input ──────────────────────────────────────────────────────────────
.num-input {
  width: 64px;
  height: 34px;
  background: #0f1014;
  border: none;
  border-radius: $radius-sm;
  color: #fff;
  font-weight: 500;
  font-size: 13px;
  letter-spacing: 0.02em;
  padding: 0 8px;
  outline: none;
  text-align: center;
  transition: box-shadow 100ms;

  &:focus { box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.3); }
  &::-webkit-inner-spin-button { -webkit-appearance: none; }
}

.res-control { gap: 6px; }
.res-x {
  font-weight: 500;
  font-size: 15px;
  color: rgba(255, 255, 255, 0.35);
}

// ── Path control ──────────────────────────────────────────────────────────────
.path-control {
  max-width: 380px;
  gap: 6px;
}

.path-text {
  font-weight: 500;
  font-size: 12.5px;
  color: rgba(255, 255, 255, 0.5);
  letter-spacing: 0.02em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 220px;
  flex-shrink: 1;
  min-width: 0;
}

.browse-btn {
  font-weight: 500;
  font-size: 12px;
  letter-spacing: 0.04em;
  padding: 8px 16px;
  background: #232328;
  border: none;
  border-radius: 12px;
  color: rgba(255, 255, 255, 0.8);
  cursor: pointer;
  transition: background 120ms, color 120ms;
  flex-shrink: 0;

  &:hover { background: #2f2f36; color: #fff; }
}

.clear-btn {
  width: 22px;
  height: 22px;
  background: transparent;
  border: 1px solid transparent;
  color: #333;
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: color 80ms, border-color 80ms;

  &:hover { color: #888; border-color: rgba(255,255,255,0.1); }
}

// ── Select ────────────────────────────────────────────────────────────────────
.mic-select { max-width: 260px; text-overflow: ellipsis; }

.mic-meter {
  width: 200px;
  height: 8px;
  background: #0f1014;
  border: none;
  border-radius: $radius-sm;
  overflow: hidden;
}

.mic-meter-fill {
  height: 100%;
  background: #30d158;
  transition: width 60ms linear;
}

.select-input {
  height: 34px;
  background: #0f1014;
  border: none;
  border-radius: $radius-sm;
  color: #fff;
  font-weight: 500;
  font-size: 13px;
  letter-spacing: 0.03em;
  padding: 0 12px;
  outline: none;
  cursor: pointer;
  appearance: none;
  min-width: 120px;
  transition: box-shadow 100ms;

  &:focus { box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.3); }
  option { background: #111; }
}

// ── Segmented control ─────────────────────────────────────────────────────────
.seg-control {
  display: flex;
  gap: 4px;
  border: none;
  border-radius: 12px;
  background: #0f1014;
  padding: 3px;
  overflow: hidden;
}

.seg-btn {
  font-weight: 500;
  font-size: 13px;
  letter-spacing: 0.04em;
  padding: 6px 14px;
  background: none;
  border: none;
  border-radius: 10px;
  color: rgba(255, 255, 255, 0.55);
  cursor: pointer;
  transition: background 100ms, color 100ms;

  &:hover:not(:disabled) { background: #1d1d22; color: #fff; }
  &.active { background: #2a2a30; color: #fff; }
  &:disabled { opacity: 0.3; cursor: not-allowed; }
}

// ── Accent color picker ───────────────────────────────────────────────────────
.accent-control {
  display: flex;
  align-items: center;
  gap: 12px;
}

.color-swatches {
  display: flex;
  gap: 5px;
  flex-wrap: wrap;
  max-width: 160px;
}

.color-swatch {
  width: 20px;
  height: 20px;
  border: 2px solid transparent;
  border-radius: $radius-sm;
  cursor: pointer;
  transition: transform 100ms, border-color 100ms;
  flex-shrink: 0;

  &:hover { transform: scale(1.15); }
  &.active { border-color: #fff; }
}

.color-custom {
  display: flex;
  align-items: center;
  gap: 7px;
  cursor: pointer;
  border: none;
  border-radius: $radius-sm;
  padding: 4px 8px;
  background: #0a0a0b;
  transition: background 100ms;

  &:hover { background: #141416; }
}

.color-input {
  width: 18px;
  height: 18px;
  padding: 0;
  border: none;
  background: none;
  cursor: pointer;
  flex-shrink: 0;
}

.color-hex {
  font-family: $font-family;
  font-weight: 600;
  font-size: 14px;
  color: #666;
  letter-spacing: 0.04em;
}
</style>
