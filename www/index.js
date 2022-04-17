import * as wasm from "test_bindgen_fetch";

wasm.run("http://localhost:7541", "toto@localhost", "experimental_counter")
	.catch(console.error)
