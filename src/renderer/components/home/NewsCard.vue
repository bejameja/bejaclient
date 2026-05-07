<template>
  <article class="news-card" @click="openLink">
    <div class="news-image">
      <img :src="imageUrl" :alt="item.title" @error="onImgError" />
      <div class="news-tag-overlay">
        <span class="news-tag">{{ item.tag }}</span>
      </div>
    </div>
    <div class="news-body">
      <span class="news-date">{{ formattedDate }}</span>
      <h3 class="news-title">{{ item.title }}</h3>
      <p class="news-text">{{ truncatedText }}</p>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { NewsEntry } from '../../types'

const MINECRAFT_IMAGES = [
  'https://staticg.sportskeeda.com/editor/2023/01/b8a7b-16740051619553-1920.jpg',
  'https://staticg.sportskeeda.com/editor/2022/12/b1c4a-16718594533098-1920.jpg',
  'https://staticg.sportskeeda.com/editor/2023/02/c9d2e-16758432157687-1920.jpg',
]
const LAST_RESORT = 'https://wallpaperaccess.com/full/1256024.jpg'

const props = defineProps<{ item: NewsEntry; index?: number }>()

const errorLevel = ref(0)

const imageUrl = computed(() => {
  const idx = (props.index ?? 0) % MINECRAFT_IMAGES.length
  if (errorLevel.value === 0 && props.item.playPageImage?.url) {
    return props.item.playPageImage.url
  }
  if (errorLevel.value <= 1) {
    return MINECRAFT_IMAGES[idx]
  }
  return LAST_RESORT
})

const formattedDate = computed(() => {
  try {
    return new Date(props.item.date).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    })
  } catch {
    return props.item.date
  }
})

const truncatedText = computed(() => {
  const text = props.item.text ?? ''
  return text.length > 90 ? text.slice(0, 90) + '...' : text
})

function openLink() {
  if (props.item.readMoreLink) {
    window.api.system.openExternal(props.item.readMoreLink)
  }
}

function onImgError() {
  if (errorLevel.value < 2) errorLevel.value++
}
</script>

<style lang="scss" scoped>
.news-card {
  width: 200px;
  background: $surface;
  border: 1px solid $border;
  border-radius: $radius;
  overflow: hidden;
  cursor: pointer;
  transition: border-color $transition;
  flex-shrink: 0;

  &:hover {
    border-color: $border-strong;
  }
}

.news-image {
  position: relative;
  width: 100%;
  height: 130px;
  overflow: hidden;
  background: $surface-elevated;

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 300ms ease;
  }
}

.news-card:hover .news-image img {
  transform: scale(1.04);
}

.news-tag-overlay {
  position: absolute;
  top: $sp-2;
  left: $sp-2;
  z-index: 1;
}

.news-tag {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: $text-primary;
  background: $surface;
  padding: 2px 6px;
  border-radius: $radius-sm;
  border: 1px solid $border;
}

.news-body {
  padding: $sp-3;
  display: flex;
  flex-direction: column;
  gap: $sp-1;
}

.news-date {
  font-size: 10px;
  color: $muted;
}

.news-title {
  font-size: 12px;
  font-weight: 700;
  color: $text-primary;
  line-height: 1.4;
}

.news-text {
  font-size: 11px;
  color: $text-secondary;
  line-height: 1.5;
}
</style>
