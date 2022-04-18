import * as wasm from "test_bindgen_fetch";

wasm.run("http://localhost:7541", "toto", "experimental_counter:rw")
	.catch(console.error)
