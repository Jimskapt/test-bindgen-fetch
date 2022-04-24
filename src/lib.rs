mod utils;

mod client;

use serde::{Deserialize, Serialize};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// Lorsque la fonctionnalité `wee_alloc` est activée, nous allons utiliser
// `wee_alloc` en tant qu'allocateur global.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const COUNTER_PATH: &str = "/experimental_counter/counter";

#[wasm_bindgen]
pub async fn run(
	webfinger_uri: String,
	user: String,
	scope: String, // TODO : make it multiple
	client_id: Option<String>,
) -> Result<(), JsValue> {
	utils::set_panic_hook();

	//////////////////////////

	let window = web_sys::window().ok_or("window not found")?;
	let document = window.document().ok_or("document not found")?;
	let document = document
		.dyn_ref::<web_sys::HtmlDocument>()
		.ok_or("can not cast document as HtmlDocument")?;

	let remote = client::ClientRemote::new(
		webfinger_uri,
		user,
		scope,
		client_id.unwrap_or(window.location().origin()?),
		true,
	)
	.await?;

	if !remote.is_connected() {
		remote.show_connect_overlay().await?;
	}

	let remote = std::sync::Arc::new(remote);

	let buttons = document.create_element("p")?;
	buttons.set_attribute("id", "buttons")?;

	let value_display = document.create_element("span")?;
	value_display.set_attribute("id", "value_display")?;
	value_display.set_inner_html(&format!("&nbsp;{}&nbsp;", 0));

	let sub = document.create_element("button")?;
	sub.set_attribute("id", "sub_button")?;
	sub.set_inner_html("-");
	let sub_value = value_trigger(-1, remote.clone())?;
	let sub = sub
		.dyn_ref::<web_sys::HtmlElement>()
		.ok_or("can not cast #sub_button as HtmlElement")?;
	sub.set_onclick(Some(sub_value.as_ref().unchecked_ref()));
	sub_value.forget();
	buttons.append_child(sub)?;

	buttons.append_child(&value_display)?;

	let add = document.create_element("button")?;
	add.set_attribute("id", "add_button")?;
	add.set_inner_html("+");
	let add_value = value_trigger(1, remote.clone())?;
	let add = add
		.dyn_ref::<web_sys::HtmlElement>()
		.ok_or("can not cast #add_button as HtmlElement")?;
	add.set_onclick(Some(add_value.as_ref().unchecked_ref()));
	add_value.forget();
	buttons.append_child(add)?;

	document
		.body()
		.ok_or("body not found")?
		.append_child(&buttons)?;

	update_counter_value(remote).ok();

	Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct WebfingerResponse {
	links: Vec<Link>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Link {
	href: String,
	properties: std::collections::HashMap<String, Option<String>>,
}

fn update_counter_value(remote: std::sync::Arc<client::ClientRemote>) -> Result<(), JsValue> {
	if remote.is_connected() {
		let window = web_sys::window().ok_or("window not found")?;
		let document = window.document().ok_or("document not found")?;

		let promise = remote.get_document(COUNTER_PATH, None)?;

		let value_display = document
			.get_element_by_id("value_display")
			.ok_or("can not found #value_display")?;

		let process_callback = Closure::once(Box::new(move |resp: JsValue| {
			let doc = resp.into_serde::<client::Document>().unwrap();

			let body = &[
				*doc.get_content().get(0).unwrap(),
				*doc.get_content().get(1).unwrap(),
				*doc.get_content().get(2).unwrap(),
				*doc.get_content().get(3).unwrap(),
			];

			let value = isize::from_be_bytes(*body);

			value_display.set_inner_html(&format!("&nbsp;{}&nbsp;", value));
		}) as Box<dyn FnOnce(JsValue)>);

		let err_callback = Closure::wrap(Box::new(move |err: JsValue| {
			web_sys::console::error_1(&format!("{:?}", err).into());
		}) as Box<dyn FnMut(JsValue)>);

		promise.then(&process_callback).catch(&err_callback);

		process_callback.forget();
		err_callback.forget();

		Ok(())
	} else {
		Err(JsValue::from_str("database connection not established"))
	}
}

fn value_trigger(
	increment: i8,
	remote: std::sync::Arc<client::ClientRemote>,
) -> Result<Closure<dyn FnMut()>, JsValue> {
	let window = web_sys::window().ok_or("window not found")?;
	let document = window.document().ok_or("document not found")?;

	Ok(Closure::wrap(Box::new(move || {
		if remote.is_connected() {
			let document = document.dyn_ref::<web_sys::HtmlDocument>().unwrap();

			let val: String = document
				.get_element_by_id("value_display")
				.unwrap_or_else(|| {
					let res = document.create_element("span").unwrap();
					res.set_inner_html("&nbsp;0&nbsp;");
					res
				})
				.text_content()
				.unwrap_or_else(|| String::from("0"));
			let val = val.trim().parse::<isize>().unwrap_or_default() + increment as isize;

			let save = remote
				.put_document(COUNTER_PATH, &client::Document::from(val))
				.unwrap();

			let remote_for_callback = remote.clone();
			let save_callback = Closure::wrap(Box::new(move |_: JsValue| {
				update_counter_value(remote_for_callback.clone()).ok();
			}) as Box<dyn FnMut(JsValue)>);
			let err_callback = Closure::wrap(Box::new(move |err: JsValue| {
				web_sys::console::error_1(&format!("{:?}", err).into())
			}) as Box<dyn FnMut(JsValue)>);

			save.then(&save_callback).catch(&err_callback);

			save_callback.forget();
			err_callback.forget();
		}
	})))
}
