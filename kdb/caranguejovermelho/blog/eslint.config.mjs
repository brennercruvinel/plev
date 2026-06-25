// flat config (eslint 9+). the static scripts are plain browser scripts loaded
// via <script>, not es modules, so sourceType is "script". vendored libraries
// (fuse, katex, search bundles) are ignored, we do not own their style.
import js from "@eslint/js";
import globals from "globals";

export default [
	{
		ignores: [
			"public/**",
			"static/*.min.js",
			"static/fuse.js",
			"static/katex.min.js",
			"static/katex-init.js",
			"static/auto-render.min.js",
			"static/search-fuse.js",
			"static/search-elasticlunr.js",
		],
	},
	js.configs.recommended,
	{
		files: ["static/**/*.js", "content/**/*.js"],
		languageOptions: {
			ecmaVersion: 2022,
			sourceType: "script",
			globals: { ...globals.browser },
		},
		rules: {
			"no-unused-vars": "warn",
			"no-var": "warn",
			"prefer-const": "warn",
		},
	},
	{
		files: ["static/sw.js", "static/register-sw.js"],
		languageOptions: {
			globals: { ...globals.browser, ...globals.serviceworker },
		},
	},
];
