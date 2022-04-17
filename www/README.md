

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
