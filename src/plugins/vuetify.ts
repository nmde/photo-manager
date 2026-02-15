import '@mdi/font/css/materialdesignicons.css';
import 'vuetify/styles';
import colors from 'vuetify/util/colors';
import { createVuetify } from 'vuetify';
import { VDateInput } from 'vuetify/labs/VDateInput';
import { VCalendar } from 'vuetify/labs/VCalendar';

export default createVuetify({
  theme: {
    defaultTheme: 'EpicTheme',
    themes: {
      EpicTheme: {
        dark: true,
        colors: {
          primary: colors.green.base,
        },
      },
    },
  },
  components: {
    VDateInput,
    VCalendar,
  },
});
