# Programming with IronCalc

This folder contains used to deploy https://ironcalc.dev

* https://design.ironcalc.dev you can find the storybook
* https://ironcalc.dev/rust you can find the rust documentation (also in docs.rs)
* https://ironcalc.dev/python the python bindings documentation
* https://ironcalc.dev/wasm holds the wasm bindings (browser) documentation
* https://ironcalc.dev/nodejs holds the nodejs documentation

## Landing page sources

Run `./build-docs.sh` to assemble everything into `dist/`.

* `index.html`, `styles.css`: the landing page. The styles follow the same design
  system as https://www.ironcalc.com
* `fonts.css`, `fonts/`: self hosted Inter (body) and Mozilla Headline (headings)
* `images/`: the logos of the languages and tools shown in the cards, which
  belong to their respective owners. The IronCalc logo, icon and favicons are
  not kept here: `build-docs.sh` copies them into `dist/images/` from the
  `assets/` folder at the root of the repository
