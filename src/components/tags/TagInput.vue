<script setup lang="ts">
  import { get_tags } from '@/api/tags';
  import { Tag, type TagData, type TagRec } from '@/classes/Tag';
  import { useFileStore } from '@/stores/fileStore';

  const props = defineProps<{
    id?: string;
    label: string;
    value: TagData['name'][];
    loading?: boolean;
    single?: boolean;
    validation?: string;
    disabled?: boolean;
    filtered?: boolean;
  }>();

  const emit = defineEmits<{
    (e: 'change', tags: TagData['name'][]): void;
    (e: 'focused', value: boolean): void;
  }>();

  const { reportError } = useFileStore();

  const tags = ref<TagRec>({});
  const localValue = ref<TagData['name'][]>([]);

  const tagColors = computed(() => {
    const colorMap: Record<string, { color: string }> = {};
    for (const tag of Object.values(tags.value)) {
      colorMap[tag.name] = { color: tag.color ?? '' };
    }
    return colorMap;
  });

  const filteredTags = computed(() => {
    if (!props.filtered) {
      return tags.value;
    }
    const result: TagRec = {};
    for (const tag of Object.values(tags.value)) {
      let allPrereqsMet = true;
      for (const prereq of tag.prereqs) {
        allPrereqsMet = allPrereqsMet && localValue.value.includes(prereq);
      }
      if (allPrereqsMet) {
        result[tag.name] = tag;
      }
    }
    return result;
  });

  async function initialize() {
    await get_tags()
      .ok(t => {
        tags.value = t;
        localValue.value = props.value;
      })
      .err(reportError)
      .send();
  }

  onMounted(initialize);

  watch([() => props.value, () => props.id], () => {
    if (props.value !== localValue.value) {
      initialize();
    }
  });
</script>

<template>
  <sorted-combo
    :id="id"
    chips
    color-key="name"
    :color-repo="tagColors"
    :disabled="disabled"
    :error-messages="validation"
    item-key="name"
    :items="filteredTags"
    :label="label"
    :loading="loading"
    :multiple="single !== true"
    :value="localValue"
    @focused="val => emit('focused', val)"
    @update="
      newTags => {
        for (const tag of newTags) {
          if (!tags[tag]) {
            tags[tag] = Tag.default(tag);
          }
        }
        localValue = newTags;
        emit('change', newTags);
      }
    "
  />
</template>
