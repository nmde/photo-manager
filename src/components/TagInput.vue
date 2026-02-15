<script setup lang="ts">
  import { get_tags } from '@/api/tags';
  import { Tag } from '@/classes/Tag';

  defineProps<{
    label: string;
    value: string[];
    loading?: boolean;
  }>();

  const emit = defineEmits<{
    (e: 'change', tags: string[]): void;
  }>();

  const tags = ref<Record<string, Tag>>({});

  const tagColors = computed(() => {
    const colorMap: Record<string, { color: string }> = {};
    for (const tag of Object.values(tags.value)) {
      colorMap[tag.name] = { color: tag.color };
    }
    return colorMap;
  });

  onMounted(async () => {
    tags.value = await get_tags();
  });
</script>

<template>
  <sorted-combo
    color-key="name"
    :color-repo="tagColors"
    item-key="name"
    :items="tags"
    :label="label"
    :loading="loading"
    multiple
    :value="value"
    @update="
      newTags => {
        for (const tag of newTags) {
          if (!tags[tag]) {
            tags[tag] = Tag.default(tag);
          }
        }
        emit('change', newTags as string[]);
      }
    "
  />
</template>
