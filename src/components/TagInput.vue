<script setup lang="ts">
  import type { Photo } from '@/classes/Photo';
  import { invoke } from '@tauri-apps/api/core';
  import { Tag, type TagData } from '@/classes/Tag';

  const props = defineProps<{
    label: string;
    value: string[];
    single?: boolean;
    filtered?: boolean;
    validate?: boolean;
    advanced?: boolean;
    target?: Photo;
    loading?: boolean;
  }>();

  const emit = defineEmits<{
    (e: 'change', tags: string[]): void;
  }>();

  const selected = ref<string[]>([]);
  const hasChanged = ref(false);
  const tags = ref<Tag[]>([]);

  async function initialize() {
    selected.value = props.value;
    tags.value = Tag.createTags(
      Object.values(await invoke<Record<string, TagData>>('get_tags')),
    ).toSorted((a, b) => b.count - a.count);
  }

  watch(
    () => props.value,
    () => initialize(),
  );

  onMounted(() => initialize());
</script>

<template>
  <v-combobox
    v-model="selected"
    chips
    clearable
    color="primary"
    :error="validate && target ? !target.valid : undefined"
    :error-messages="validate && target ? target.validationMessage : undefined"
    item-title="name"
    item-value="name"
    :items="tags"
    :label="props.label"
    :loading="loading"
    multiple
    @update:model-value="
      (values: any[]) => {
        hasChanged = true;
        // If the user types a tag, the value is a string. If the user CLICK a tag, it's a Tag.
        // I think this is an issue with Vuetify
        let fixed: string[] = values.map(val => typeof val === 'string' ? val : (val as Tag).name);
        const last = fixed[fixed.length - 1];
        if (single && last) {
          fixed = [last];
        }
        selected = fixed;
        emit('change', selected);
      }
    "
  >
    <template #item="{ item, props: lprops }">
      <v-list-item
        v-bind="lprops"
        :style="{ color: item.raw.color }"
        :title="`${item.title} (${item.raw.count})`"
      />
    </template>
    <template #chip="{ item, props: cprops }">
      <v-chip v-bind="cprops" :color="tags.find(t => t.name === item.value)?.color" />
    </template>
  </v-combobox>
  <div v-if="advanced" />
</template>
