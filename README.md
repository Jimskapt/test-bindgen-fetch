
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
