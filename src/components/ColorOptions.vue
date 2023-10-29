<script setup lang="ts">
const emit = defineEmits<{
  (e: 'select', color: string): void;
}>();

const colors = [
  '#F44336',
  '#E91E63',
  '#9C27B0',
  '#673AB7',
  '#3F51B5',
  '#2196F3',
  '#03A9F4',
  '#00BCD4',
  '#009688',
  '#4CAF50',
  '#8BC34A',
  '#CDDC39',
  '#FFEB3B',
  '#FFC107',
  '#FF9800',
  '#FF5722',
];
</script>

<template>
  <div class="color-opts">
    <div class="color-opt clear-opt" @click="emit('select', '')">
      <v-icon>mdi-close-circle-outline</v-icon>
    </div>
    <div
      v-for="color in colors"
      :key="color"
      class="color-opt"
      :style="{ 'background-color': color }"
      @click="emit('select', color)"
    ></div>
  </div>
</template>

<style scoped>
.color-opts {
  display: flex;
  height: 50px;
}

.color-opt {
  cursor: pointer;
  width: 100px;
}

.clear-opt {
  display: flex;
  justify-content: center;
  align-items: center;
}
</style>
