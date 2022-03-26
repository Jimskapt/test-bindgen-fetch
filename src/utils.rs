pub fn set_panic_hook() {
	// Lorsque la fonctionnalité `console_error_panic_hook` est activée, nous
	// pouvons faire appel à la fonction `set_panic_hook` au moins une fois à
	// l'initialisation afin d'obtenir de meilleurs messages d'erreur si notre
	// code panique.
	//
	// Pour en savoir plus, rendez-vous à :
	// https://github.com/rustwasm/console_error_panic_hook#readme
	#[cfg(feature = "console_error_panic_hook")]
	console_error_panic_hook::set_once();
}
