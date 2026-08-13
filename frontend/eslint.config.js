import js from '@eslint/js';
import pluginVue from 'eslint-plugin-vue';
import tseslint from 'typescript-eslint';
import globals from 'globals';
import eslintConfigPrettier from '@vue/eslint-config-prettier/skip-formatting';

export default [
	{
		ignores: ['dist/', 'node_modules/', 'static/favicon/']
	},
	js.configs.recommended,
	...tseslint.configs.recommended,
	...pluginVue.configs['flat/recommended'],
	{
		files: ['**/*.{ts,vue}'],
		languageOptions: {
			globals: {
				...globals.browser,
				...globals.node
			},
			parserOptions: {
				parser: tseslint.parser
			}
		},
		rules: {
			'vue/multi-word-component-names': 'off'
		}
	},
	eslintConfigPrettier
];
