import antfu from '@antfu/eslint-config'

export default antfu(
  {
    formatters: false,
    perfectionist: false,
    imports: false,
    e18e: false,
    regexp: false,
    ignores: ['**/components/ui/**', '**/vite-env.d.ts', 'vite.config.ts', 'tsconfig.json'],
    stylistic: false,
    javascript: {
      overrides: {
        'no-console': 'off',
      },
    },
    typescript: {
      overrides: {
        'ts/consistent-type-definitions': 'off',
        'ts/no-use-before-define': 'off',
      },
    },
    vue: {
      stylistic: false,
      overrides: {
        // Keep Vue's correctness and template checks without making this toolchain
        // migration reformat existing templates.
        'vue/attributes-order': 'off',
        'vue/block-tag-newline': 'off',
        'vue/first-attribute-linebreak': 'off',
        'vue/html-closing-bracket-newline': 'off',
        'vue/html-indent': 'off',
        'vue/html-self-closing': 'off',
        'vue/multiline-html-element-content-newline': 'off',
        'vue/singleline-html-element-content-newline': 'off',
        'vue/comma-dangle': 'off',
        'vue/prefer-template': 'off',
      },
    },
  },
)
