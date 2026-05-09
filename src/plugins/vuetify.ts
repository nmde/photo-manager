import { createVuetify } from 'vuetify';
import { VDateInput } from 'vuetify/labs/VDateInput';
import '@mdi/font/css/materialdesignicons.css';
import 'vuetify/styles';

export const vuetify = createVuetify({
  theme: {
    defaultTheme: 'Theme',
    themes: {
      Theme: {
        dark: true,
        colors: {
          // Observatory Slate — the single accent (oklch 63% 0.09 245)
          primary: '#5f87ac',
          'on-primary': '#f0f2f5',
          // Lifted Slate — toolbar backgrounds, secondary actions (oklch 18% 0.003 245)
          secondary: '#22262a',
          'on-secondary': '#e2e4e8',
          // Deep Space — application canvas (oklch 9% 0.003 245)
          background: '#0d1013',
          'on-background': '#e2e4e8',
          // Surface — card and panel backgrounds (oklch 14% 0.003 245)
          surface: '#1a1d20',
          'on-surface': '#e2e4e8',
          'surface-variant': '#22262a',
          'on-surface-variant': '#969aa0',
          // Signal Red — error states only (oklch 58% 0.13 14)
          error: '#c56070',
          'on-error': '#f0f2f5',
        },
      },
      LightTheme: {
        dark: false,
        colors: {
          primary: '#3d6882',
          'on-primary': '#ffffff',
        },
      },
    },
  },
  components: {
    VDateInput,
  },
});
