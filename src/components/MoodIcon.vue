<script setup lang="ts">
import { moods } from '../stores/fileStore';

const props = defineProps<{
  mood: number;
}>();

const emit = defineEmits<{
  (e: 'selected', mood: number): void;
}>();
</script>

<template>
  <v-menu>
    <template v-slot:activator="{ props }">
      <v-btn v-bind="props" icon flat :style="{ color: moods[mood].color }">
        <v-icon v-if="mood === 0">mdi-emoticon-dead-outline</v-icon>
        <v-icon v-if="mood === 1">mdi-emoticon-sad-outline</v-icon>
        <v-icon v-if="mood === 2">mdi-emoticon-neutral-outline</v-icon>
        <v-icon v-if="mood === 3">mdi-emoticon-happy-outline</v-icon>
        <v-icon v-if="mood === 4">mdi-emoticon-outline</v-icon>
      </v-btn>
    </template>
    <v-list>
      <v-list-item
        v-for="mood in moods"
        :key="mood.value"
        :style="{ color: mood.color }"
        @click="emit('selected', mood.value)"
        >{{ mood.label }}</v-list-item
      >
    </v-list>
  </v-menu>
</template>
