// Un graphe de dépendance contenant un wasm qui doit être importé de manière
// asynchrone. Ce fichier `bootstrap.js` procède à un seul import asynchrone,
// pour ne plus s'en préoccuper ensuite.
import("./index.js")
  .catch(e => console.error("Erreur d'import de `index.js`:", e));
