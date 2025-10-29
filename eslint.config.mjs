import vuetify from 'eslint-config-vuetify';
import { defineConfig } from 'eslint/config';

export default defineConfig(await vuetify(), {
  languageOptions: {
    parserOptions: {
      projectService: true,
      tsconfigRootDir: import.meta.dirname,
    },
  },
  rules: {
    '@stylistic/semi': ['warn', 'always'],
    '@stylistic/member-delimiter-style': [
      'warn',
      {
        multiline: {
          delimiter: 'semi',
          requireLast: true,
        },
      },
    ],
    '@stylistic/space-before-function-paren': ['warn', { anonymous: 'always', named: 'never' }],
    'unicorn/prefer-event-target': 'off',
  },
});
