<script setup lang="ts">
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
    <template v-slot:activator="{ props }">
      <v-btn v-bind="props" flat icon :color="tmpColor"></v-btn>
    </template>
    <v-color-picker
      v-model="tmpColor"
      @update:model-value="() => emit('update', tmpColor)"
    ></v-color-picker>
  </v-menu>
</template>

<style scoped>
.color-indicator {
  width: 50px;
  height: 50px;
}
</style>
