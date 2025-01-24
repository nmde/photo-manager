<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';

const props = defineProps<{
  label?: string;
  value: string;
}>();

const emit = defineEmits<{
  (e: 'save', text: string): void;
}>();

const local = ref('');

let timer: NodeJS.Timeout;
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
  <v-textarea :label="label" v-model="local" @update:model-value="handleKeypress()"></v-textarea>
</template>
