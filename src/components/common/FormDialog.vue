<script setup lang="ts">
  const model = defineModel<boolean>({ required: true });

  defineProps<{
    title: string;
  }>();

  const emit = defineEmits<{
    (e: 'submit'): void;
  }>();
</script>

<template>
  <v-dialog v-model="model" max-width="80vw">
    <v-card :title="title">
      <v-form
        validate-on="input"
        @submit.prevent="
          async event => {
            if (await event) {
              emit('submit');
            }
          }
        "
      >
        <v-card-text><slot /></v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="model = false">Cancel</v-btn>
          <v-btn color="primary" type="submit">Save</v-btn>
        </v-card-actions>
      </v-form>
    </v-card>
  </v-dialog>
</template>
