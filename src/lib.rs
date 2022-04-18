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

const OAUTH_KEY: &str = "http://tools.ietf.org/html/rfc6749#section-4.2";
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

	let window = web_sys::window().unwrap();
	let document = window.document().ok_or("document not found")?;
	let document = document.dyn_ref::<web_sys::HtmlDocument>().unwrap();
	let body = document.body().ok_or("body not found")?;

	let app_block = document.create_element("div")?;
	app_block.set_attribute("id", "app_block")?;
	body.append_child(&app_block)?;

	document
		.set_cookie(&format!(
			"counter_path={}",
			pct_str::PctString::encode(COUNTER_PATH.chars(), pct_str::URIReserved)
		))
		.unwrap();

	let client = client::Client::new(
		webfinger_uri,
		user,
		scope,
		client_id.unwrap_or(format!("{}", window.location().origin().unwrap())),
		true,
	)
	.await
	.unwrap();
	let client = std::sync::Arc::new(client);

	let document = document.dyn_ref::<web_sys::HtmlDocument>().unwrap();

	let buttons = document.create_element("p")?;
	buttons.set_attribute("id", "buttons").unwrap();
	let buttons = buttons.dyn_ref::<web_sys::HtmlElement>().unwrap();

	app_block.append_child(buttons).unwrap();

	let value_display = document.create_element("span").unwrap();
	value_display.set_attribute("id", "value_display").unwrap();
	value_display.set_inner_html(&format!("&nbsp;{}&nbsp;", 0));

	let buttons = document.get_element_by_id("buttons").unwrap();

	let sub = document.create_element("button").unwrap();
	sub.set_inner_html("-");
	let sub_value = value_trigger(-1, client.clone());
	let sub = sub.dyn_ref::<web_sys::HtmlElement>().unwrap();
	sub.set_onclick(Some(sub_value.as_ref().unchecked_ref()));
	sub_value.forget();
	buttons.append_child(&sub).unwrap();

	buttons.append_child(&value_display).unwrap();

	let add = document.create_element("button").unwrap();
	add.set_inner_html("+");
	let add_value = value_trigger(1, client.clone());
	let add = add.dyn_ref::<web_sys::HtmlElement>().unwrap();
	add.set_onclick(Some(add_value.as_ref().unchecked_ref()));
	add_value.forget();
	buttons.append_child(&add).unwrap();

	update_counter_value(client.clone()).ok();

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

fn update_counter_value(client: std::sync::Arc<client::Client>) -> Result<(), JsValue> {
	let window = web_sys::window().ok_or("window not found")?;
	let document = window.document().ok_or("document not found")?;
	let document = document.dyn_ref::<web_sys::HtmlDocument>().unwrap();
	let all_cookies = document.cookie().unwrap();

	let mut counter_path = None;
	for cookie in all_cookies.split(';') {
		let mut iter = cookie.split('=');
		let name = iter.next().map(str::trim);
		let value = iter
			.next()
			.map(|res| pct_str::PctString::new(res.trim()).unwrap().decode());

		if let Some("counter_path") = name {
			counter_path = value;
		}
	}

	if let Some(counter_path) = counter_path {
		let promise = client.get_document(counter_path, None);

		if let Ok(promise) = promise {
			let process_callback = Closure::once(Box::new(move |resp: JsValue| {
				let resp: web_sys::Response = resp.dyn_into().unwrap();

				if resp.ok() {
					let body_process = Closure::wrap(Box::new(move |body: JsValue| {
						let body = js_sys::ArrayBuffer::from(body);
						let body = js_sys::DataView::new(&body, 0, 4);
						let body = &[
							body.get_uint8(0),
							body.get_uint8(1),
							body.get_uint8(2),
							body.get_uint8(3),
						];

						let value = isize::from_be_bytes(*body);

						let window = web_sys::window().expect("window not found");
						let document = window.document().expect("document not found");

						let value_display = document.get_element_by_id("value_display").unwrap();
						value_display.set_inner_html(&format!("&nbsp;{}&nbsp;", value));
					}) as Box<dyn FnMut(JsValue)>);

					let body_err = Closure::wrap(Box::new(move |err: JsValue| {
						web_sys::console::error_1(&format!("{:?}", err).into());
					}) as Box<dyn FnMut(JsValue)>);

					resp.array_buffer()
						.unwrap()
						.then(&body_process)
						.catch(&body_err);
					body_process.forget();
					body_err.forget();
				} else if resp.status() == 404 {
					web_sys::console::error_1(
						&format!("value does not exists yet in database\n",).into(),
					);
				} else {
					web_sys::console::error_1(
						&format!("error {} when access to database\n", resp.status()).into(),
					);
				}
			}) as Box<dyn FnOnce(JsValue)>);

			let err_callback = Closure::wrap(Box::new(move |err: JsValue| {
				web_sys::console::error_1(&format!("{:?}", err).into());
			}) as Box<dyn FnMut(JsValue)>);

			promise.then(&process_callback).catch(&err_callback);

			process_callback.forget();
			err_callback.forget();
		}
	}

	Ok(())
}

fn value_trigger(increment: i8, client: std::sync::Arc<client::Client>) -> Closure<dyn FnMut()> {
	Closure::wrap(Box::new(move || {
		let window = web_sys::window().unwrap();
		let document = window.document().expect("document not found");

		let val: String = document
			.get_element_by_id("value_display")
			.unwrap_or_else(|| {
				let res = document.create_element("span").unwrap();
				res.set_inner_html("&nbsp;0&nbsp;");
				res
			})
			.text_content()
			.unwrap_or_else(|| String::from("0"));
		let val = val.trim().parse::<isize>().unwrap_or_default() + 1 * increment as isize;

		let document = document.dyn_ref::<web_sys::HtmlDocument>().unwrap();
		let all_cookies = document.cookie().unwrap();

		let mut counter_path = None;
		for cookie in all_cookies.split(';') {
			let mut iter = cookie.split('=');
			let name = iter.next().map(str::trim);
			let value = iter
				.next()
				.map(|res| pct_str::PctString::new(res.trim()).unwrap().decode());

			if let Some("counter_path") = name {
				counter_path = value;
			}
		}

		if let Some(counter_path) = counter_path {
			let save = client
				.put_document(counter_path, &client::Document::from(val))
				.unwrap();

			let client_for_callback = client.clone();
			let save_callback = Closure::wrap(Box::new(move |_: JsValue| {
				update_counter_value(client_for_callback.clone()).ok();
			}) as Box<dyn FnMut(JsValue)>);
			let err_callback = Closure::wrap(Box::new(move |err: JsValue| {
				web_sys::console::error_1(&format!("{:?}", err).into())
			}) as Box<dyn FnMut(JsValue)>);
			save.then(&save_callback).catch(&err_callback);
			save_callback.forget();
			err_callback.forget();
		}
	}))
}
