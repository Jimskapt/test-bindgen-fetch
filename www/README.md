*(🇬🇧 🇺🇸 it's the french translation of the npm package create-wasm-app)*

<div align="center">

  <h1><code>create-wasm-app-fr</code></h1>

  <strong>Un modèle <code>npm init</code> pour démarrer rapidement un projet
  qui utilise des paquets NPM contenant du WebAssembly généré par Rust et les
  regroupe avec Webpack.</strong>

  <p>
    <a href="https://travis-ci.org/Jimskapt/create-wasm-app-fr">
      <img src="https://img.shields.io/travis/Jimskapt/create-wasm-app-fr.svg?style=flat-square" alt="Build Status" />
    </a>
  </p>

  <h3>
    <a href="#usage">Utilisation</a>
    <span> | </span>
    <a href="https://discordapp.com/channels/442252698964721669/443151097398296587">Tchat (EN)</a>
  </h3>

  <sub>Construit avec 🦀🕸 par <a href="https://rustwasm.github.io/">le groupe de travail Rust et WebAssembly</a></sub>
</div>

## A propos

Ce modèle est conçu pour dépendre de paquets NPM qui contiennent du WebAssembly
générés par Rust et les utiliser pour créer un site Web.

* Vous souhaitez créer un paquet NPM avec Rust et WebAssembly ? [Jettez un oeil
  à `wasm-pack-template-fr`.](https://github.com/Jimskapt/wasm-pack-template-fr)
* Vous souhaitez créer un site web en un seul dépôt sans le publier sur NPM ?
  Penchez-vous sur
  [`rust-webpack-template`](https://github.com/rustwasm/rust-webpack-template)
  et / ou
  [`rust-parcel-template`](https://github.com/rustwasm/rust-parcel-template).

## 🚴 Utilisation

```
npm init wasm-app-fr
```

## 🔋 Piles incluses

- `.gitignore` : ignore le `node_modules`
- `LICENSE-APACHE` et `LICENSE-MIT` : la plupart des projets Rust sont publiés
  selon ces licences, donc nous les avons inclus pour vous
- `README.md` : le fichier que vous êtes actuellement en train de regarder !
- `index.html` : un document html brut qui inclut le paquet webpack
- `index.js` : un fichier exemple en JavaScript avec un commentaire qui montre
  comment importer et utiliser un paquet wasm
- `package.json` et `package-lock.json` :
  - ils importent les dépendances de développement pour utiliser webpack :
      - [`webpack`](https://www.npmjs.com/package/webpack)
      - [`webpack-cli`](https://www.npmjs.com/package/webpack-cli)
      - [`webpack-dev-server`](https://www.npmjs.com/package/webpack-dev-server)
  - ils définissent un script `start` pour lancer `webpack-dev-server`
- `webpack.config.js` : un fichier de configuration pour regrouper vos js avec
  webpack

## Licence

Publié sous licence double :

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

... selon votre convenance.

### Contribution

Sauf indication contraire explicite de votre part, toute contribution que vous
soumettrez intentionnellement pour être incluse dans ce projet, tel que défini
dans la licence Apache-2.0, fera l'objet d'une double licence comme ci-dessus,
sans conditions supplémentaires.
