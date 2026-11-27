// ESLint flat config (frontend.md §1 Lint row, §8): type-aware typescript-eslint, React hooks, and formatjs
// `no-literal-string-in-jsx` so literal UI text in JSX or in label-like props fails CI (I-8).
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import formatjs from "eslint-plugin-formatjs";
import reactHooks from "eslint-plugin-react-hooks";
import globals from "globals";

export default tseslint.config(
  {
    ignores: ["dist/", "coverage/", "test-results/", "playwright-report/", "src/api/generated/", "src/i18n/messages.ts"],
  },
  js.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: { allowDefaultProject: ["*.js", "scripts/*.mjs"] },
        tsconfigRootDir: import.meta.dirname,
      },
      globals: { ...globals.browser },
    },
  },
  {
    files: ["**/*.{ts,tsx}"],
    plugins: { formatjs, "react-hooks": reactHooks },
    rules: {
      ...reactHooks.configs.flat["recommended-latest"].rules,
      "formatjs/no-literal-string-in-jsx": [
        "error",
        {
          props: {
            include: [["*", "{label,placeholder,title,description,emptyText,removeLabel,alt}"]],
          },
        },
      ],
      "formatjs/enforce-description": "off",
      "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],
      "@typescript-eslint/consistent-type-imports": "error",
      "@typescript-eslint/no-floating-promises": "error",
    },
  },
  {
    files: ["**/*.{js,mjs}"],
    ...tseslint.configs.disableTypeChecked,
  },
  {
    // Tests, e2e and config files may use literal strings (selectors, fixtures) and node globals.
    files: ["**/*.test.{ts,tsx}", "src/test/**", "e2e/**", "*.config.ts", "scripts/**", "*.js"],
    languageOptions: { globals: { ...globals.node } },
    rules: {
      "formatjs/no-literal-string-in-jsx": "off",
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "@typescript-eslint/no-unsafe-call": "off",
      "@typescript-eslint/no-unsafe-argument": "off",
    },
  },
);
