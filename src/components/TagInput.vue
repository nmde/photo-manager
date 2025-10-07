<script setup lang="ts">
  import { computed, ref, watch } from 'vue';
  import { fileStore } from '../stores/fileStore';

  const { getTagColor, validateTags, getFile, tags, advTags } = fileStore;

  const props = defineProps<{
    label: string;
    value: string | string[];
    single?: boolean;
    filtered?: boolean;
    validate?: string;
    advanced?: boolean;
    target?: string;
  }>();

  const emit = defineEmits<{
    (e: 'change' | 'update', tags: string[]): void;
  }>();

  const selected = ref<string[]>([]);
  const valid = ref<boolean | undefined>(true);
  const validationMsg = ref<string | undefined>(undefined);
  const hasChanged = ref(false);

  const targetPhoto = computed(() => (props.target ? getFile(props.target) : undefined));

  const filteredTags = computed(() => {
    if (!props.filtered) {
      return tags;
    }
    const filtered: string[] = [];
    for (const tag of tags) {
      if (props.value.includes(tag)) {
        // Always show tags that are already enabled regardless of prereqs
        filtered.push(tag);
        continue;
      }
      const a = advTags.find(t => t.name === tag);
      if (a) {
        if (a.prereqs.length > 0) {
          let anyPrereqMet = false;
          for (const p of a.prereqs) {
            anyPrereqMet = anyPrereqMet || props.value.includes(p);
          }
          if (anyPrereqMet) {
            filtered.push(tag);
          }
        } else {
          filtered.push(tag);
        }
      } else {
        filtered.push(tag);
      }
    }
    return filtered;
  });

  /**
   * Validates the tags.
   */
  function validateTagsWrapper() {
    if (props.validate) {
      validateTags(props.validate);
    }
  }

  function initialize() {
    selected.value = typeof props.value === 'string' ? [props.value] : props.value;
    if (props.validate) {
      validateTagsWrapper();
    }
  }

  watch(() => props.value, initialize);

  fileStore.on('validationUpdate', () => {
    valid.value = targetPhoto.value?.valid;
    validationMsg.value = targetPhoto.value?.validationMsg;
  });
</script>

// The global sorted tag list is not updating when new tags are added, links are created
<template>
  <v-combobox
    v-model="selected"
    chips
    clearable
    :error="!valid"
    :error-messages="validationMsg"
    :items="filteredTags"
    :label="props.label"
    :multiple="props.single ? false : true"
    @update:focused="
      () => {
        if (hasChanged) {
          emit('update', selected);
          validateTagsWrapper();
        }
      }
    "
    @update:model-value="
      () => {
        hasChanged = true;
        emit('change', selected);
        validateTagsWrapper();
      }
    "
  >
    <template #item="{ item, props: lprops }">
      <v-list-item v-bind="lprops" :style="{ color: getTagColor(item.title) }" />
    </template>
    <template #chip="{ item, props: cprops }">
      <v-chip v-bind="cprops" :color="getTagColor(item.title)" />
    </template>
  </v-combobox>
  <div v-if="advanced" />
</template>
