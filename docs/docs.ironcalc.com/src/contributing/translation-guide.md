---
layout: doc
outline: deep
lang: en-US
---

# Translating IronCalc

Thanks for your interest in bringing IronCalc to more people! This guide explains what adding a language involves, and what you should consider before starting work.

::: warning
**Please note we're not accepting new languages at the moment.** If you have a strong case for adding one, please open a [Discussion](https://github.com/ironcalc/IronCalc/discussions) and tell us about it before sending a translation pull request. Every language we add has to be updated each time IronCalc changes (which, trust us, it's quite often), and our team can only review a few languages _with confidence_. Because of this, translation pull requests opened without prior discussion are likely to be closed, even when the work done is good.
:::

## Why translations are hard for us

IronCalc currently supports 5 languages: **English**, **Spanish**, **French**, **German** and **Italian**. We translate new strings for all of them with every release, thanks to the help of AI tools first, and then reviewing the results ourselves. That only works because the team can understand these languages _well enough_ to spot mistakes.

When a new language arrives to IronCalc, we take on a commitment that lasts much longer than the PR:

- **Every new feature needs new strings.** Someone has to translate them, every time, so the language doesn't fall behind and leave the interface half translated.
- **We can't review what we can't understand.** If nobody on the team knows the language or its script, we can't tell a good translation from a bad one. We also don't know how good AI tools are at translating certain languages.
- **Bugs can show up in unexpected places.** A new language often exposes assumptions in the code (input handling, text direction, fonts), and those take time to fix properly.

We would love to add more languages, but each one needs planning, and ideally someone who can help us maintain it.

## A language is more than the UI

Translating IronCalc touches three separate layers, and new proposals need to take care of each of them in order to be complete.

### 1. Interface

This covers menus, buttons, dialogs, tooltips and messages... These strings live in JSON files (one per language), in two places:

- `webapp/IronCalc/src/locale/` for the spreadsheet widget.
- `webapp/app.ironcalc.com/frontend/src/locale/` for the web application around it.

Each file needs to be registered in the `i18n.ts` next to it.

### 2. Engine language

IronCalc translates formulas, just like Excel does. In Spanish you write `=SI(A1>0;"sí";"no")` instead of `=IF(A1>0,"yes","no")`. The engine language defines:

- **Function names**, such as `SUM` becoming `SUMA`.
- **Booleans**, such as `TRUE` and `FALSE`.
- **Error values**, such as `#VALUE!` or `#NAME?`.
- **The default sheet name**, such as `Sheet1` or `Hoja1`.

This data lives in `generate_language/src/languages.json` and is compiled into `base/src/language/language.bin`. Default sheet names are set in `base/src/new_empty.rs`.

::: tip
**Do not assume function names should be translated!** This needs research for every language, most of the times it's not even clear for us. Excel translates function names in some languages (Spanish, German, French) but keeps them in English in others (Japanese, for instance). We follow what Excel does in each language, because that's what users already know and what their existing files and tutorials use. Include a source for your choice in the Discussion.
:::

### 3. Locale

The locale controls how values look, and it's independent from the language. Two people who both speak English can write the same number differently. The locale defines:

- **Decimal and thousands separators**: `1,234.56` or `1.234,56`, for example.
- **The formula argument separator**: languages that use a comma as decimal separator usually separate arguments with a semicolon.
- **Currency**: which symbol is used, and whether it goes before or after the number (`$10` or `10 €`).
- **Dates and times**: day and month names, and short and long date formats (`12/15/2025`, `15/12/2025` or `15.12.2025`).

We generate this data from [Unicode CLDR](https://cldr.unicode.org/) instead of writing it by hand. See `generate_locale/README.md` for how to regenerate it. The output is `base/src/locale/locales.bin`. Users can also change the locale per workbook from [Regional Settings](/features/regional-settings).

## Scripts, fonts and text direction

IronCalc uses the [Inter](https://rsms.me/inter/) font. Inter covers the Latin, Greek and Cyrillic alphabets, but we currently only ship its Latin subset, so Greek and Cyrillic would need extra font files. Other scripts, such as Thai, Armenian, Hebrew, Arabic, Devanagari, Chinese, Japanese or Korean, aren't covered by Inter and fall back to whatever font the user's system provides. The result may be readable, but it won't match the rest of the interface, and it can affect how text fits in cells and menus.

Right-to-left (RTL) languages, like Hebrew, Arabic or Persian, raise more questions:

- The interface is not mirrored, so menus and panels stay in left-to-right order, which may not be ideal.
- Mixing RTL text with numbers and formulas in a cell needs careful handling.
- CLDR data for these locales contains invisible direction marks that must not end up in number formats.

Supporting a new script takes design and engineering work on our side. If your language needs it, mention it in your Discussion so we can plan it together.

## Before you propose a language

When you open a Discussion, please include as much of this as you can:

1. **Language and region**, for example *Portuguese (Brazil)* or *Portuguese (Portugal)*.
2. **Function names**: does Excel translate them in this language? Link a source.
3. **Booleans and errors**: are they translated in Excel?
4. **Locale details**: decimal separator, thousands separator, argument separator, currency symbol and its position, and the usual date format.
5. **Script and direction**: is it written in the Latin alphabet? Is it RTL?
6. **Maintenance**: can you (or someone you know) review translations for future releases?

The last point is key, as a potential language, with a native speaker willing to review updates, is much easier for us to accept.

## If we agree to add a language

Once we've agreed on a plan, a PR will usually touch:

- The interface JSON files and both `i18n.ts` files.
- The `LANGUAGES` lists in `components/Navigation/FileMenu.tsx` and `components/Navigation/MobileMenu.tsx`, so the language shows up in the menus.
- `webapp/app.ironcalc.com/frontend/src/components/storage.ts`, which detects the browser language, keeps the saved preference and maps it to a locale code. Without this, the choice is lost on reload and the locale falls back to English.
- The locale-to-language mapping in `RegionalSettings.tsx`.
- `generate_language/src/languages.json` and the regenerated `language.bin`.
- `generate_locale/locales_list.json` and the regenerated `locales.json` and `locales.bin`.
- The default sheet names in `base/src/new_empty.rs`.
- Tests in `base/src/test/test_locale.rs` covering formulas, numbers and dates in the new language.
- The list of supported languages in the [Language Selector](/web-application/language-selector) page.

Keep the PR focused on the language. Changes to unrelated files make it much harder to review, and will likely imply a rejection.

If you have questions at any point, ask us on [Discord](https://discord.com/invite/zZYWfh3RHJ) or in your Discussion. Looking forward to seeing what you can bring!

Thank you · Gracias · Merci · Danke · Grazie · Obrigado · Mulțumesc · Dziękuję · Tack · Дякую · Спасибо · Ευχαριστώ · شكرًا · תודה · धन्यवाद · ありがとう · 谢谢 · 감사합니다
