import vuetify from 'eslint-config-vuetify';
import { defineConfig } from 'eslint/config';
import tseslint from 'typescript-eslint';

// eslint-disable-next-line no-restricted-exports

export default defineConfig(
  tseslint.configs.strictTypeChecked,

  await vuetify(),

  {
    languageOptions: {
      parserOptions: {
        projectService: true,

        tsconfigRootDir: import.meta.dirname,
      },
    },

    rules: {
      '@stylistic/indent': 'off',

      '@stylistic/semi': ['warn', 'always'],

      '@stylistic/member-delimiter-style': [
        'warn',
        {
          multiline: {
            delimiter: 'semi',
          },
        },
      ],

      '@stylistic/indent-binary-ops': ['warn', 2],

      '@stylistic/space-before-function-paren': 'off',

      'unicorn/no-nested-ternary': 'off',

      '@stylistic/jsx-wrap-multilines': 'off',

      '@stylistic/multiline-ternary': 'off',

      '@stylistic/quote-props': ['warn', 'as-needed'],

      'unicorn/prefer-event-target': 'off',

      '@typescript-eslint/no-invalid-this': 'off',

      '@stylistic/jsx-one-expression-per-line': 'off',

      'no-restricted-exports': [
        'warn',
        {
          restrictDefaultExports: {
            direct: true,
          },
        },
      ],

      '@stylistic/operator-linebreak': 'off',

      '@stylistic/jsx-curly-newline': [
        'warn',
        {
          multiline: 'consistent',
        },
      ],

      'vue/script-indent': ['warn', 2, {
        baseIndent: 0,
        switchCase: 1,
      }],

      'vue/html-self-closing': 'off',
    },
  },
);
