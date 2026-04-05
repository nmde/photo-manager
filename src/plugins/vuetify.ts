import { createVuetify } from 'vuetify';
import { VDateInput } from 'vuetify/labs/VDateInput';
import colors from 'vuetify/util/colors';
import '@mdi/font/css/materialdesignicons.css';
import 'vuetify/styles';

export const vuetify = createVuetify({
  theme: {
    defaultTheme: 'Theme',
    themes: {
      Theme: {
        dark: true,
        colors: {
          primary: colors.green.base,
        },
      },
      LightTheme: {
        dark: false,
        colors: {
          primary: colors.green.base,
        },
      },
    },
  },
  components: {
    VDateInput,
  },
});
