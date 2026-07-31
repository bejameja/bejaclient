<template>
  <Tooltip :text="text" :placement="placement">
    <img :src="infoIcon" alt="" class="info-tooltip-icon" draggable="false" />
  </Tooltip>
</template>

<script setup lang="ts">
import Tooltip from './Tooltip.vue'
import infoIcon from '../../assets/icons/info-icon.png'

withDefaults(
  defineProps<{ text: string; placement?: 'top' | 'bottom' | 'left' | 'right' }>(),
  { placement: 'top' },
)
</script>

<style lang="scss" scoped>
.info-tooltip-icon {
  width: 15px;
  height: 15px;
  vertical-align: middle;
  cursor: help;
  opacity: 0.85;
  transition: opacity $transition;

  &:hover {
    opacity: 1;
  }
}
</style>
