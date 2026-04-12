<script setup lang="ts">
  defineProps<{
    message?: string;
  }>();
</script>

<template>
  <span v-if="message !== undefined && message.length > 0" class="error-message">{{
    message
  }}</span>
</template>

<style scoped>
  .error-message {
    color: rgb(var(--v-theme-error));
    margin-top: 12px;
    margin-left: 16px;
  }
</style>
