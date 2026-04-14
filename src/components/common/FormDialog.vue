<script setup lang="ts">
  const model = defineModel<boolean>({ required: true });

  defineProps<{
    title: string;
    saveText?: string;
    reset: () => void;
  }>();

  const saving = ref(false);

  const emit = defineEmits<{
    (e: 'submit'): Promise<void>;
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
              saving = true;
              await emit('submit');
              saving = false;
              reset();
              model = false;
            }
          }
        "
      >
        <v-card-text><slot /></v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="model = false">Cancel</v-btn>
          <v-btn color="primary" :loading="saving" type="submit">{{ saveText ?? 'Save' }}</v-btn>
        </v-card-actions>
      </v-form>
    </v-card>
  </v-dialog>
</template>
