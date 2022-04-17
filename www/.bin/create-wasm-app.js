#!/usr/bin/env node

const { spawn } = require("child_process");
const fs = require("fs");

let nomDossier = '.';

if (process.argv.length >= 3) {
  nomDossier = process.argv[2];
  if (!fs.existsSync(nomDossier)) {
    fs.mkdirSync(nomDossier);
  }
}

const clone = spawn("git", ["clone", "https://github.com/Jimskapt/create-wasm-app-fr.git", nomDossier]);

clone.on("close", code => {
  if (code !== 0) {
    console.error("le clonage du modèle a échoué !")
    process.exit(code);
  } else {
    console.log("🦀 Rust + 🕸 Wasm = ❤");
  }
});
