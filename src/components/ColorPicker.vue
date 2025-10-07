<script setup lang="ts">
  import { onMounted, ref } from 'vue';

  const props = defineProps<{
    color: string;
  }>();

  const emit = defineEmits<{
    (e: 'update', color: string): void;
  }>();

  const tmpColor = ref('');

  onMounted(() => {
    tmpColor.value = props.color;
  });
</script>

<template>
  <v-menu>
    <template #activator="{ props: bprops }">
      <v-btn v-bind="bprops" :color="tmpColor" flat icon />
    </template>
    <v-color-picker v-model="tmpColor" @update:model-value="() => emit('update', tmpColor)" />
  </v-menu>
</template>

<style scoped>
  .color-indicator {
    width: 50px;
    height: 50px;
  }
</style>
