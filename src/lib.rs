mod utils;

use serde::{Deserialize, Serialize};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

// Lorsque la fonctionnalité `wee_alloc` est activée, nous allons utiliser
// `wee_alloc` en tant qu'allocateur global.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const OAUTH_KEY: &str = "http://tools.ietf.org/html/rfc6749#section-4.2";

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
	let body = document.body().ok_or("body not found")?;

	let app_block = document.create_element("div")?;
	app_block.set_attribute("id", "app_block")?;
	body.append_child(&app_block)?;

	let status_block = document.create_element("pre")?;
	status_block.set_attribute("id", "status_block")?;
	let status_block = status_block.dyn_ref::<web_sys::HtmlElement>().unwrap();
	status_block
		.style()
		.set_property("border", "1px solid black")?;
	status_block.style().set_property("padding", "0.5em")?;
	status_block.style().set_property("overflow", "auto")?;
	body.append_child(&status_block)?;

	//////////////////////////

	lazy_static::lazy_static! {
		static ref ACCESS_TOKEN_REGEX: regex::Regex = regex::Regex::new("^#.*access_token=([^&]+).+$").unwrap();
	}

	let access_token = {
		let hash = window.location().hash();

		match hash {
			Ok(hash) => {
				if hash.contains("token_type") && ACCESS_TOKEN_REGEX.is_match(&hash) {
					if let Some(matches) = ACCESS_TOKEN_REGEX.captures_iter(&hash).next() {
						matches.get(1).map(|access_token| {
							pct_str::PctString::new(access_token.as_str())
								.unwrap()
								.decode()
						})
					} else {
						None
					}
				} else {
					None
				}
			}
			Err(_) => None,
		}
	};

	match access_token {
		Some(access_token) => {
			status_block.set_inner_html(&format!(
				"{}found token {access_token}\n",
				status_block.inner_html()
			));

			let buttons = document.create_element("p")?;
			let buttons = buttons.dyn_ref::<web_sys::HtmlElement>().unwrap();

			let webfinger = fetch_webfinger_content(webfinger_uri, user).await?;
			let path = webfinger.links[0].href.clone();
			let counter_path = format!("{path}/experimental_counter/counter");

			let value = {
				let mut opts = RequestInit::new();
				opts.method("GET");
				opts.mode(RequestMode::Cors);

				status_block.set_inner_html(&format!(
					"{}fetch {counter_path} ... ",
					status_block.inner_html()
				));

				let request = Request::new_with_str_and_init(&counter_path, &opts)?;
				request
					.headers()
					.set("Authorization", &format!("Bearer {access_token}"))?;

				let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

				status_block.set_inner_html(&format!("{}done\n", status_block.inner_html()));

				let resp: Response = resp_value.dyn_into().unwrap();

				std::sync::Arc::new(std::sync::Mutex::new(if resp.ok() {
					let body = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
					let body_for_callback = body.clone();
					JsFuture::from(resp.text().unwrap().then(&Closure::wrap(Box::new(move |res: JsValue| {
						*body_for_callback.lock().unwrap() = res.as_string().unwrap();
					}) as Box<dyn FnMut(JsValue)>))).await?;
					let body = body.lock().unwrap();
					body.parse::<isize>().unwrap()
				} else {
					status_block.set_inner_html(&format!(
						"{}value does not exists yet in database\n",
						status_block.inner_html()
					));
					0isize
				}))
			};

			let value_display = document.create_element("span")?;
			value_display.set_attribute("id", "value_display")?;
			value_display.set_inner_html(&format!("&nbsp;{}&nbsp;", value.lock().unwrap()));

			let sub = document.create_element("button")?;
			sub.set_inner_html("-");
			let sub_value = Closure::wrap(value_trigger(
				value.clone(),
				-1,
				counter_path.clone(),
				access_token.clone(),
			));
			let sub = sub.dyn_ref::<web_sys::HtmlElement>().unwrap();
			sub.set_onclick(Some(sub_value.as_ref().unchecked_ref()));
			sub_value.forget();
			buttons.append_child(&sub)?;

			buttons.append_child(&value_display)?;

			let add = document.create_element("button")?;
			add.set_inner_html("+");
			let add_value = Closure::wrap(value_trigger(
				value.clone(),
				1,
				counter_path.clone(),
				access_token.clone(),
			));
			let add = add.dyn_ref::<web_sys::HtmlElement>().unwrap();
			add.set_onclick(Some(add_value.as_ref().unchecked_ref()));
			add_value.forget();
			buttons.append_child(&add)?;

			app_block.append_child(buttons)?;
		}
		None => {
			let response = fetch_webfinger_content(webfinger_uri, user).await?;

			status_block.set_inner_html(&format!(
				"{}found {} link{}{}<br>",
				status_block.inner_html(),
				response.links.len(),
				if response.links.len() == 1 { "" } else { "s" },
				if !response.links.is_empty() {
					" :"
				} else {
					"."
				}
			));

			for (i, link) in response.links.iter().enumerate() {
				status_block.set_inner_html(&format!(
					"{}- {}: {}<br>",
					status_block.inner_html(),
					i + 1,
					link.href
				));
			}

			let search_oauth = response
				.links
				.iter()
				.enumerate()
				.find(|(_, link)| matches!(link.properties.get(OAUTH_KEY), Some(Some(_))));

			match search_oauth {
				Some((link_id, link)) => {
					let client_id =
						client_id.unwrap_or(format!("{}", window.location().to_string()));

					let scope = format!("{scope}:rw"); // TODO : make it custom

					let oauth_path = link.properties.get(OAUTH_KEY).unwrap().as_ref().unwrap();
					let oauth_path = format!(
						"{oauth_path}?redirect_uri={}&scope={}&client_id={}&response_type={}",
						pct_str::PctString::encode(
							format!("{}", window.location().to_string()).chars(),
							pct_str::URIReserved
						), // TODO : change to base url (no page name, or its arguments)
						pct_str::PctString::encode(scope.chars(), pct_str::URIReserved),
						pct_str::PctString::encode(client_id.chars(), pct_str::URIReserved),
						pct_str::PctString::encode("token".chars(), pct_str::URIReserved),
					);

					status_block.set_inner_html(&format!(
						"{}using link #{} :<br>{}",
						status_block.inner_html(),
						link_id + 1,
						oauth_path
					));

					let button = document.create_element("button")?;
					let button = button.dyn_ref::<web_sys::HtmlElement>().unwrap();
					button.set_inner_html("GO NEXT &gt");

					let change_location = Closure::wrap(Box::new(move || {
						window.location().set_href(&oauth_path).unwrap();
					}) as Box<dyn FnMut()>);
					button.set_onclick(Some(change_location.as_ref().unchecked_ref()));
					change_location.forget();

					app_block.append_child(button)?;
				}
				None => {
					status_block.set_inner_html(&format!(
						"{}no oauth links found, stop everything",
						status_block.inner_html()
					));
				}
			}
		}
	}

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

fn value_trigger(
	value: std::sync::Arc<std::sync::Mutex<isize>>,
	increment: i8,
	counter_path: impl Into<String>,
	access_token: impl Into<String>,
) -> Box<dyn FnMut()> {
	let counter_path = counter_path.into();
	let access_token = access_token.into();

	Box::new(move || {
		let mut val = value.lock().unwrap();
		*val = *val + 1 * increment as isize;

		let status_block = web_sys::window()
			.unwrap()
			.document()
			.expect("document not found")
			.get_element_by_id("status_block")
			.unwrap();

		status_block.set_inner_html(&format!(
			"{}new value : {}\n",
			status_block.inner_html(),
			val
		));

		web_sys::window()
			.unwrap()
			.document()
			.expect("document not found")
			.get_element_by_id("value_display")
			.unwrap()
			.set_inner_html(&format!("&nbsp;{}&nbsp;", val));

		let mut opts = RequestInit::new();
		opts.method("PUT");
		opts.body(Some(&format!("{}", val).into()));
		opts.mode(RequestMode::Cors);

		status_block.set_inner_html(&format!(
			"{}PUT {counter_path} ... ",
			status_block.inner_html()
		));

		let request = Request::new_with_str_and_init(&counter_path, &opts).unwrap();
		request
			.headers()
			.set("Authorization", &format!("Bearer {access_token}"))
			.unwrap();
		request
			.headers()
			.set("Content-Type", "text/plain")
			.unwrap();

		/*let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

		status_block.set_inner_html(&format!("{}done\n", status_block.inner_html()));

		let resp: Response = resp_value.dyn_into().unwrap();

		status_block.set_inner_html(&format!("{}{}\n", status_block.inner_html(), if resp.ok() { "OK" } else { "ERR" }));*/

		web_sys::window().unwrap().fetch_with_request(&request);
		status_block.set_inner_html(&format!("{}done\n", status_block.inner_html()));

		web_sys::console::log_1(&format!("new value : {}", val).into());
	})
}

async fn fetch_webfinger_content(
	webfinger_uri: impl Into<String>,
	user: impl Into<String>,
) -> Result<WebfingerResponse, JsValue> {
	let webfinger_uri = webfinger_uri.into();
	let user = user.into();

	let window = web_sys::window().unwrap();

	let status_block = window
		.document()
		.expect("document not found")
		.get_element_by_id("status_block")
		.unwrap();

	status_block.set_inner_html(&format!(
		"{}connection to {webfinger_uri} with user {user} ... ",
		status_block.inner_html()
	));

	let mut opts = RequestInit::new();
	opts.method("GET");
	opts.mode(RequestMode::Cors);

	let webfinger_uri = webfinger_uri.strip_suffix('/').unwrap_or(&webfinger_uri);

	let url = format!("{webfinger_uri}/.well-known/webfinger?resource=acct:{user}");

	status_block.set_inner_html(&format!("{}{url} ... ", status_block.inner_html()));

	let request = Request::new_with_str_and_init(&url, &opts)?;

	let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

	status_block.set_inner_html(&format!("{}done\n", status_block.inner_html()));

	let resp: Response = resp_value.dyn_into().unwrap();
	let json = JsFuture::from(resp.json()?).await?;
	let response: WebfingerResponse = json.into_serde().unwrap();

	Ok(response)
}
