<script setup lang="ts">
import { moods } from '../stores/fileStore';

const props = defineProps<{
  mood: number;
}>();
</script>

<template>
  <div :style="{ color: moods[mood].color }">
    <div v-if="mood === 0">
      <v-icon>mdi-emoticon-dead-outline</v-icon>
    </div>
    <div v-if="mood === 1">
      <v-icon>mdi-emoticon-sad-outline</v-icon>
    </div>
    <div v-if="mood === 2">
      <v-icon>mdi-emoticon-neutral-outline</v-icon>
    </div>
    <div v-if="mood === 3">
      <v-icon>mdi-emoticon-happy-outline</v-icon>
    </div>
    <div v-if="mood === 4">
      <v-icon>mdi-emoticon-outline</v-icon>
    </div>
  </div>
</template>
