<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useFileStore } from '../stores/fileStore';

const fileStore = useFileStore();
const { getTagColor, validateTags } = fileStore;
const { tags, advTags } = storeToRefs(fileStore);

const props = defineProps<{
  label: string;
  value: string | string[];
  single?: boolean;
  filtered?: boolean;
  validate?: string;
}>();

const emit = defineEmits<{
  (e: 'change', tags: string[]): void;
  (e: 'update', tags: string[]): void;
}>();

const selected = ref<string[]>([]);
const valid = ref(true);
const validationMsg = ref('');
const focused = ref(false);

const filteredTags = computed(() => {
  if (!props.filtered) {
    return tags.value;
  }
  const filtered: string[] = [];
  tags.value.forEach((tag) => {
    if (props.value.indexOf(tag) >= 0) {
      // Always show tags that are already enabled regardless of prereqs
      filtered.push(tag);
      return;
    }
    const a = advTags.value.find((t) => t.data.name === tag);
    if (a) {
      if (a.prereqs.length > 0) {
        let anyPrereqMet = false;
        a.prereqs.forEach((p) => {
          anyPrereqMet = anyPrereqMet || props.value.indexOf(p) >= 0;
        });
        if (anyPrereqMet) {
          filtered.push(tag);
        }
      } else {
        filtered.push(tag);
      }
    } else {
      filtered.push(tag);
    }
  });
  return filtered;
});

/**
 * Validates the tags.
 */
function validateTagsWrapper() {
  if (props.validate) {
    const msg = validateTags(props.validate);
    if (msg) {
      valid.value = false;
      validationMsg.value = msg;
    } else {
      valid.value = true;
      validationMsg.value = '';
    }
  }
}

function initialize() {
  if (typeof props.value === 'string') {
    selected.value = [props.value];
  } else {
    selected.value = props.value;
  }
  if (props.validate) {
    validateTagsWrapper();
  }
}

watch(() => props.value, initialize);
</script>

<template>
  <v-combobox
    :label="props.label"
    :items="filteredTags"
    :multiple="props.single ? false : true"
    chips
    clearable
    v-model="selected"
    @update:model-value="
      () => {
        emit('change', selected);
        if (valid) {
          validateTagsWrapper();
        }
      }
    "
    @update:focused="
      () => {
        if (focused) {
          emit('update', selected);
        }

        focused = !focused;
      }
    "
    :error="!valid"
    :error-messages="validationMsg"
  >
    <template v-slot:item="{ item, props }">
      <v-list-item v-bind="props" :style="{ color: getTagColor(item.title) }"></v-list-item>
    </template>
    <template v-slot:chip="{ item, props }">
      <v-chip v-bind="props" :color="getTagColor(item.title)"></v-chip>
    </template>
  </v-combobox>
</template>
