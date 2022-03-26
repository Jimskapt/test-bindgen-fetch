<div align="center">

  <h1><code>wasm-pack-template-fr</code></h1>

  <p>
    <strong>Un modèle pour générer rapidement un projet en Rust et WebAssembly en utilisant <a href="https://github.com/rustwasm/wasm-pack">wasm-pack</a>.</strong>
  </p>

  <p>
    <i>(this the french translation of <a href="https://github.com/rustwasm/wasm-pack-template">https://github.com/rustwasm/wasm-pack-template</a>)</i>
  </p>

  <p>
    <a href="https://travis-ci.org/Jimskapt/wasm-pack-template-fr">
      <img src="https://img.shields.io/travis/Jimskapt/wasm-pack-template-fr.svg?style=flat-square" alt="Build Status" />
    </a>
  </p>

  <h3>
    <a href="https://rustwasm.github.io/docs/wasm-pack/tutorials/npm-browser-packages/index.html">Tutoriel wasm-pack (EN)</a>
    <span> | </span>
    <a href="https://discordapp.com/channels/442252698964721669/443151097398296587">Tchat (EN)</a>
  </h3>

  <sub>Construit avec 🦀🕸 par <a href="https://rustwasm.github.io/">le groupe de travail de Rust et WebAssembly</a></sub>
</div>

## A propos

[**📚 Lisez ce tutoriel sur le modèle (EN) ! 📚**][template-docs]

Ce modèle est conçu pour compiler des bibliothèques Rust en WebAssembly et
pour publier le paquet qui en résulte sur NPM.

Ne ratez pas [les autres tutoriels sur `wasm-pack` en ligne][tutorials] pour
découvrir d'autres modèles et cas d'utilisation de `wasm-pack`.

[tutorials]: https://rustwasm.github.io/docs/wasm-pack/tutorials/index.html
[template-docs]: https://rustwasm.github.io/docs/wasm-pack/tutorials/npm-browser-packages/index.html

## 🚴 Utilisation

### 🐑 Utilisez `cargo generate` pour cloner ce modèle

[En apprendre plus sur `cargo generate` ici.](https://github.com/ashleygwilliams/cargo-generate)

```
cargo generate --git https://github.com/Jimskapt/wasm-pack-template-fr.git --name mon-projet
cd mon-projet
```

### 🛠️ Compiler avec `wasm-pack build`

```
wasm-pack build
```

### 🔬 Tester dans un navigateur sans tête avec `wasm-pack test`

```
wasm-pack test --headless --firefox
```

### 🎁 Publier sur NPM avec `wasm-pack publish`

```
wasm-pack publish
```

## 🔋 Piles incluses

* [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) pour communiquer
  entre WebAssembly et JavaScript.
* [`console_error_panic_hook`](https://github.com/rustwasm/console_error_panic_hook)
  pour journaliser les erreurs de panic dans la console de développement.
* [`wee_alloc`](https://github.com/rustwasm/wee_alloc), un allocateur optimisé
  pour avoir un poids minime.
