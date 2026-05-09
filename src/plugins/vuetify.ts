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
          // Observatory Slate — the single accent (oklch 65% 0.14 245)
          primary: '#4b86bb',
          'on-primary': '#f0f2f5',
          // Lifted Slate — toolbar backgrounds, secondary actions (oklch 21% 0.006 245)
          secondary: '#22262a',
          'on-secondary': '#e7edf4',
          // Deep Space — application canvas (oklch 9% 0.003 245)
          background: '#0d1013',
          'on-background': '#e7edf4',
          // Surface — card and panel backgrounds (oklch 14% 0.004 245)
          surface: '#1a1d20',
          'on-surface': '#e7edf4',
          'surface-variant': '#282f38',
          'on-surface-variant': '#8a9098',
          // Signal Red — error states only (oklch 58% 0.13 14)
          error: '#c56070',
          'on-error': '#f0f2f5',
          // Amber — warning states: duplicate flag, rating stars (oklch 72% 0.16 60)
          warning: '#eb882e',
          'on-warning': '#0d1013',
        },
      },
      LightTheme: {
        dark: false,
        colors: {
          primary: '#2e6490',
          'on-primary': '#ffffff',
        },
      },
    },
  },
  components: {
    VDateInput,
  },
});
