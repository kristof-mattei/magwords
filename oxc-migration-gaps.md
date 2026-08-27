# Lost in the ESLint/Prettier to oxlint/oxfmt migration

oxlint 1.79.0, oxlint-tsgolint 7.0.2001, oxfmt 0.64.0 (2026-08). Anything not listed is matched or exceeded.

## Linter

| Gap                                                                                                         | From               | Covered by                                | Revisit when                                                                                                      |
| ----------------------------------------------------------------------------------------------------------- | ------------------ | ----------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `@typescript-eslint/naming-convention`                                                                      | eslint-config-love | nothing                                   | tsgolint implements it                                                                                            |
| Markdown code-block linting                                                                                 | own config         | nothing                                   | oxc-project/oxc#18407                                                                                             |
| `import-x/no-relative-packages`                                                                             | own config         | nothing, dependency-cruiser can if needed | oxlint adds it                                                                                                    |
| `unicorn/class-reference-in-static-methods`                                                                 | own config         | nothing                                   | oxlint adds it; was configured with `preferThis: false`, `preferSuper: false`, upstream defaults are the opposite |
| `unicorn/consistent-boolean-name`                                                                           | unicorn all preset | nothing                                   | oxlint adds it                                                                                                    |
| `unicorn/no-keyword-prefix`                                                                                 | unicorn all preset | nothing                                   | oxlint adds it                                                                                                    |
| eslint-comments: `disable-enable-pair`, `no-aggregating-enable`, `no-duplicate-disable`, `no-unused-enable` | eslint-config-love | nothing                                   | oxc-project/oxc#22193 reopens                                                                                     |

## Deliberately off, not yet in oxlint

If oxlint adds these, the categories turn them on; re-disable:

| Rule                                        | Why off                                                 |
| ------------------------------------------- | ------------------------------------------------------- |
| `unicorn/consistent-class-member-order`     | conflicts with `perfectionist/sort-classes`             |
| `unicorn/no-declarations-before-early-exit` | const goes at the top                                   |
| `unicorn/prefer-minimal-ternary`            | ignores types                                           |
| `unicorn/prefer-type-literal-last`          | `perfectionist/sort-intersection-types` owns type order |

## Formatter

`prettier-plugin-sh` was commented out of the Prettier config, so shell scripts were never formatted. To format shell scripts, use shfmt directly (its settings sit in `.editorconfig`): oxfmt has no shell support and cannot load Prettier plugins.
