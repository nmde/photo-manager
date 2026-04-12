<script setup lang="ts">
  import type { Nullable } from '@/types';
  import { add_color, get_colors, promote_color } from '@/api/settings';
  import { useFileStore } from '@/stores/fileStore';

  const store = useFileStore();
  const { reportError } = store;

  const color = defineModel<Nullable<string> | undefined>({ required: true });

  const props = defineProps<{
    disabled?: boolean;
    error?: boolean;
  }>();

  const size = 16;
  const dialog = ref(false);
  const newColor = ref('');
  const colors = ref<string[]>([]);

  function setColor(value: Nullable<string>) {
    if (!props.disabled) {
      color.value = value;
      if (value !== null && colors.value.indexOf(value) >= size) {
        promote_color(value)
          .ok(c => (colors.value = c))
          .err(reportError)
          .send();
      }
    }
  }

  onMounted(async () => {
    await get_colors()
      .ok(c => (colors.value = c))
      .err(reportError)
      .send();
  });
</script>

<template>
  <div class="color-opts" :style="{ border: `2px solid ${color}` }">
    <div
      :class="{ 'color-opt clear-opt': true, 'color-opt--disabled': disabled }"
      @click="setColor(null)"
    >
      <v-icon>mdi-close-circle-outline</v-icon>
    </div>
    <div
      v-for="c in colors.slice(0, size)"
      :key="c"
      :class="{ 'color-opt': true, 'color-opt--disabled': disabled }"
      :style="{ 'background-color': c }"
      @click="setColor(c)"
    />
    <div
      :class="{ 'color-opt clear-opt': true, 'color-opt--disabled': disabled }"
      @click="
        () => {
          if (!disabled) {
            dialog = true;
          }
        }
      "
    >
      <v-icon>mdi-plus</v-icon>
    </div>
  </div>
  <error-hint :message="error ? 'A color is required.' : undefined" />
  <v-dialog v-model="dialog" :max-width="1000">
    <v-card title="Color Options">
      <v-card-text>
        <v-container>
          <v-row>
            <v-col cols="8">
              <div class="color-opts">
                <div
                  v-for="c in colors"
                  :key="c"
                  class="color-opt"
                  :style="{ 'background-color': c }"
                  @click="setColor(c)"
                />
              </div>
            </v-col>
            <v-col cols="4">
              {{ newColor }}
              <v-color-picker v-model="newColor" />
              <br />
              <v-btn
                color="primary"
                @click="
                  async () => {
                    await add_color(newColor)
                      .ok(c => (colors = c))
                      .err(reportError)
                      .send();
                  }
                "
              >
                Add New Color
              </v-btn>
            </v-col>
          </v-row>
        </v-container>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<style scoped>
  .color-opts {
    display: flex;
    flex-wrap: wrap;
    border-radius: 2px;
  }

  .color-opt {
    cursor: pointer;
    height: 50px;
    width: 5.5%;
  }

  .clear-opt {
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .color-opt--disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
</style>
