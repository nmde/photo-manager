<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';

const props = defineProps<{
  label?: string;
  value: string;
}>();

const emit = defineEmits<{
  (e: 'save', text: string): void;
}>();

const local = ref('');

let timer: number;
function handleKeypress() {
  clearTimeout(timer);
  timer = setTimeout(() => {
    emit('save', local.value);
  }, 500);
}

watch(props, () => {
  local.value = props.value;
});

onMounted(() => {
  local.value = props.value;
});
</script>

<template>
  <v-textarea v-model="local" :label="label" @update:model-value="handleKeypress()" />
</template>
