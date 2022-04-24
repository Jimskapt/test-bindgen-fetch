use js_sys::Promise;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

lazy_static::lazy_static! {
	static ref ACCESS_TOKEN_REGEX: regex::Regex = regex::Regex::new("^#.*access_token=([^&]+).+$").unwrap();
}

const OAUTH_KEY: &str = "http://tools.ietf.org/html/rfc6749#section-4.2";

pub struct ClientRemote {
	webfinger_root_uri: String,
	username: String,
	scope: String,
	client_id: String,
	pub debug: bool, // TODO
	client: Option<Client>,
}
impl ClientRemote {
	pub async fn new(
		webfinger_root_uri: impl Into<String>,
		username: impl Into<String>,
		scope: impl Into<String>,
		client_id: impl Into<String>,
		debug: bool,
	) -> Result<Self, JsValue> {
		let webfinger_root_uri = webfinger_root_uri.into();
		let username = username.into();
		let scope = scope.into();
		let client_id = client_id.into();

		////////////////////////

		let mut result = Self {
			webfinger_root_uri,
			username,
			scope,
			client_id,
			debug,
			client: None, // TODO
		};

		result.try_mount_saved_client().await?;

		Ok(result)
	}
}
impl ClientRemote {
	pub fn get_document(
		&self,
		path: impl Into<String>,
		etag: Option<String>, // TODO
	) -> Result<Promise, JsValue> {
		match &self.client {
			Some(client) => client.get_document(path, etag),
			None => Err(JsValue::from_str("client is not connected")),
		}
	}
	pub fn put_document(
		&self,
		path: impl Into<String>,
		document: &Document,
	) -> Result<Promise, JsValue> {
		match &self.client {
			Some(client) => client.put_document(path, document),
			None => Err(JsValue::from_str("client is not connected")),
		}
	}
}
impl ClientRemote {
	fn generate_cookie_name_header(&self) -> String {
		let webfinger_root_uri_obj = self.webfinger_root_uri.parse::<http::uri::Uri>().unwrap();
		let client_id_uri_obj = self.client_id.parse::<http::uri::Uri>().unwrap();

		let cookie_name_header = format!(
			"{}|{}|{}|{}|",
			match client_id_uri_obj.port() {
				Some(port) => format!("{}:{}", client_id_uri_obj.host().unwrap(), port),
				None => String::from(client_id_uri_obj.host().unwrap()),
			},
			self.username,
			match webfinger_root_uri_obj.port() {
				Some(port) => format!("{}:{}", webfinger_root_uri_obj.host().unwrap(), port),
				None => String::from(webfinger_root_uri_obj.host().unwrap()),
			},
			self.scope
		);
		let cookie_name_header =
			pct_str::PctString::encode(cookie_name_header.chars(), pct_str::URIReserved);

		cookie_name_header.to_string()
	}
	async fn try_get_webfinger_data(&self) -> Result<WebfingerResponse, JsValue> {
		let mut opts = web_sys::RequestInit::new();
		opts.method("GET");
		opts.mode(web_sys::RequestMode::Cors);

		let webfinger_uri = self
			.webfinger_root_uri
			.strip_suffix('/')
			.unwrap_or(&self.webfinger_root_uri);

		let webfinger_root_uri_obj = self.webfinger_root_uri.parse::<http::uri::Uri>().unwrap();

		let url = format!(
			"{webfinger_uri}/.well-known/webfinger?resource=acct:{}@{}",
			self.username,
			webfinger_root_uri_obj.host().unwrap()
		);

		let request = web_sys::Request::new_with_str_and_init(&url, &opts)?;

		let window = web_sys::window().ok_or_else(|| JsValue::from_str("window not found"))?;

		let resp_value =
			wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?; // TODO : try to remove await
		let resp: web_sys::Response = resp_value.dyn_into()?;
		let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?; // TODO : try to remove await
		let response: WebfingerResponse = json.into_serde().unwrap();

		Ok(response)
	}
	async fn try_mount_saved_client(&mut self) -> Result<bool, JsValue> {
		let window = web_sys::window().ok_or_else(|| JsValue::from_str("window not found"))?;
		let document = window
			.document()
			.ok_or_else(|| JsValue::from_str("document not found"))?;
		let document = document
			.dyn_ref::<web_sys::HtmlDocument>()
			.ok_or_else(|| JsValue::from_str("document can not be casted into HtmlDocument"))?;

		let hash = window.location().hash()?;

		let access_token = if hash.contains("token_type") && ACCESS_TOKEN_REGEX.is_match(&hash) {
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
		};

		let access_token = match access_token {
			Some(access_token) => {
				// hide token from URL
				window
					.history()?
					.replace_state_with_url(&String::new().into(), "", Some("/"))?;

				document
					.set_cookie(&format!(
						"{}access_token={}",
						self.generate_cookie_name_header(),
						pct_str::PctString::encode(access_token.chars(), pct_str::URIReserved)
					))
					.unwrap();

				Some(access_token)
			}
			None => {
				let document = window
					.document()
					.ok_or_else(|| JsValue::from_str("document not found"))?;
				let document = document.dyn_ref::<web_sys::HtmlDocument>().unwrap();

				let all_cookies = document.cookie()?;

				let mut access_token = None;
				for cookie in all_cookies.split(';') {
					let mut iter = cookie.split('=');
					let name = iter.next().map(str::trim);
					let value = iter
						.next()
						.map(|res| pct_str::PctString::new(res.trim()).unwrap().decode());

					if let Some(name) = name {
						if name == format!("{}access_token", self.generate_cookie_name_header()) {
							access_token = value;
						}
					}
				}

				access_token
			}
		};

		match access_token {
			Some(access_token) => {
				let webfinger = self.try_get_webfinger_data().await?; // TODO : try to remove await

				let server_path = webfinger.links.get(0).map(|link| link.href.clone());

				match server_path {
					Some(server_path) => {
						let mut opts = web_sys::RequestInit::new();
						opts.method("HEAD");
						opts.mode(web_sys::RequestMode::Cors);

						let subfolder = self.scope.split(':').next().unwrap(); // TODO

						let full_path = format!("{}/{}/", server_path, subfolder);

						let request =
							web_sys::Request::new_with_str_and_init(&full_path, &opts).unwrap();
						request
							.headers()
							.set("Authorization", &format!("Bearer {}", access_token))?;

						let window = web_sys::window().ok_or("window not found")?;

						let root_head = wasm_bindgen_futures::JsFuture::from(Promise::new(
							&mut |resolve, reject| {
								let reject = std::sync::Arc::new(reject);

								let reject_for_main = reject.clone();
								let process_callback =
									Closure::once(Box::new(move |resp: JsValue| {
										let resp: web_sys::Response = resp.dyn_into().unwrap();

										if resp.ok() {
											resolve
												.call1(&JsValue::NULL, &JsValue::from_bool(true))
												.unwrap();
										} else {
											resolve
												.call1(&JsValue::NULL, &JsValue::from_bool(false))
												.unwrap();
										}
									}) as Box<dyn FnOnce(JsValue)>);

								let err_callback = Closure::wrap(Box::new(move |err: JsValue| {
									reject
										.call1(&JsValue::NULL, &format!("{:?}", err).into())
										.unwrap();
								}) as Box<dyn FnMut(JsValue)>);

								window
									.fetch_with_request(&request)
									.then(&process_callback)
									.catch(&err_callback);

								process_callback.forget();
								err_callback.forget();
							},
						))
						.await?;

						let root_head = root_head.as_bool();

						if let Some(true) = root_head {
							self.client = Some(Client {
								access_token,
								server_path,
								debug: self.debug,
							});

							Ok(true)
						} else {
							Ok(false)
						}
					}
					None => Err(JsValue::from_str(
						"can not find `links` content in webfinger reponse of the server",
					)),
				}
			}
			None => Ok(false),
		}
	}
}
impl ClientRemote {
	pub async fn show_connect_overlay(&self) -> Result<(), JsValue> {
		let webfinger = self.try_get_webfinger_data().await?; // TODO : try to remove await

		match webfinger.links.get(0) {
			Some(link) => {
				let window =
					web_sys::window().ok_or_else(|| JsValue::from_str("window not found"))?;
				let document = window
					.document()
					.ok_or_else(|| JsValue::from_str("document not found"))?;

				let oauth_origin = link.properties.get(OAUTH_KEY).unwrap().as_ref().unwrap();
				let oauth_path = format!(
					"{oauth_origin}?redirect_uri={}&scope={}&client_id={}&response_type={}",
					pct_str::PctString::encode(
						format!("{}", window.location().to_string()).chars(),
						pct_str::URIReserved
					), // TODO : change to base url (no page name, or its arguments)
					pct_str::PctString::encode(self.scope.chars(), pct_str::URIReserved),
					pct_str::PctString::encode(self.client_id.chars(), pct_str::URIReserved),
					pct_str::PctString::encode("token".chars(), pct_str::URIReserved),
				);

				// window.location().set_href(&oauth_path).unwrap();

				let next_window = document.create_element("div")?;
				next_window.set_attribute("id", "pontus_onyx_oauth_next_window")?;
				let next_window = next_window.dyn_ref::<web_sys::HtmlElement>().unwrap();

				next_window
					.style()
					.set_property("border", "5px solid #FF4B03")?;
				next_window.style().set_property("background", "white")?;
				next_window.style().set_property("padding", "1em")?;
				next_window.style().set_property("text-align", "center")?;
				next_window.style().set_property("position", "absolute")?;
				next_window.style().set_property("width", "50%")?;
				next_window.style().set_property("height", "50%")?;
				next_window.style().set_property("left", "25%")?;
				next_window.style().set_property("top", "25%")?;
				next_window.style().set_property("opacity", "0.8")?;

				let svg = document.create_element("svg")?;
				svg.set_inner_html(include_str!("remoteStorage.svg"));
				let svg = svg.dyn_ref::<web_sys::HtmlElement>().unwrap();
				svg.set_attribute("width", "50")?;
				svg.set_attribute("height", "50")?;
				next_window.append_child(svg)?;

				let explain = document.create_element("p")?;
				explain.set_inner_html(
					&format!(
						r#"You will be temporary redirected to<br><a href="{}">{}</a><br>in order to authenticate on the requested remoteStorage server, then bring back to this page."#,
						oauth_path,
						oauth_origin
					)
				);

				next_window.append_child(&explain)?;

				let p_buttons = document.create_element("p")?;

				let abort = document.create_element("button")?;
				let abort = abort.dyn_ref::<web_sys::HtmlElement>().unwrap();
				abort.style().set_property("width", "40%")?;
				abort.style().set_property("height", "5em")?;
				abort.style().set_property("border", "2px solid #FF4B03")?;
				abort.style().set_property("background", "white")?;
				abort.style().set_property("cursor", "pointer")?;
				abort.style().set_property("font-weight", "bold")?;
				abort.set_inner_html("❌ Abort");
				let close_next_window = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
					if let Some(window) = web_sys::window() {
						if let Some(document) = window.document() {
							if let Some(body) = document.body() {
								if let Some(node) =
									document.get_element_by_id("pontus_onyx_oauth_next_window")
								{
									body.remove_child(&node).ok();
								}
							}
						}
					}
				})
					as Box<dyn FnMut()>);
				abort.set_onclick(Some(close_next_window.as_ref().unchecked_ref()));
				close_next_window.forget();

				p_buttons.append_child(abort)?;

				let a_next = document.create_element("a")?;
				a_next.set_attribute("href", &oauth_path)?;

				let button_next = document.create_element("button")?;
				let button_next = button_next.dyn_ref::<web_sys::HtmlElement>().unwrap();
				button_next.set_inner_html("Next &gt;");
				button_next.style().set_property("width", "40%")?;
				button_next.style().set_property("height", "5em")?;
				button_next.style().set_property("margin-left", "10%")?;
				button_next
					.style()
					.set_property("border", "2px solid black")?;
				button_next.style().set_property("background", "#FF4B03")?;
				button_next.style().set_property("cursor", "pointer")?;
				button_next.style().set_property("font-weight", "bold")?;
				a_next.append_child(button_next)?;

				p_buttons.append_child(&a_next)?;

				next_window.append_child(&p_buttons)?;

				document.body().unwrap().append_child(next_window)?;

				// TODO : automatic redirection ?

				Ok(())
			}
			None => Err(JsValue::from_str(
				"can not find `links` content in webfinger reponse of the server",
			)),
		}
	}
}
impl ClientRemote {
	pub fn is_connected(&self) -> bool {
		self.client.is_some()
	}
}

pub struct Client {
	server_path: String,
	access_token: String,
	pub debug: bool, // TODO
}
impl Client {
	pub fn get_document(
		&self,
		path: impl Into<String>,
		etag: Option<String>, // TODO
	) -> Result<Promise, JsValue> {
		let path = path.into();

		let mut opts = web_sys::RequestInit::new();
		opts.method("GET");
		opts.mode(web_sys::RequestMode::Cors);

		let full_path = format!("{}{}", self.server_path, path);

		let request = web_sys::Request::new_with_str_and_init(&full_path, &opts).unwrap();
		request
			.headers()
			.set("Authorization", &format!("Bearer {}", self.access_token))?;

		let window = web_sys::window().ok_or("window not found")?;

		Ok(Promise::new(&mut |resolve, reject| {
			let reject = std::sync::Arc::new(reject);

			let reject_for_main = reject.clone();
			let process_callback = Closure::once(Box::new(move |resp: JsValue| {
				let resp: web_sys::Response = resp.dyn_into().unwrap();

				if resp.ok() {
					let headers = resp.headers();

					let etag = headers.get("etag");
					if etag.is_err() {
						reject_for_main
							.call1(
								&JsValue::NULL,
								&JsValue::from_str(
									"can not get `Etag` header from server response",
								),
							)
							.unwrap();
					}
					let etag = etag.unwrap();

					let content_type = headers.get("content-type");
					if content_type.is_err() {
						reject_for_main
							.call1(
								&JsValue::NULL,
								&JsValue::from_str(
									"can not get `Content-Type` header from server response",
								),
							)
							.unwrap();
					}
					let content_type = content_type.unwrap();
					if content_type.is_none() {
						reject_for_main
							.call1(
								&JsValue::NULL,
								&JsValue::from_str(
									"missing `Content-Type` header from server response",
								),
							)
							.unwrap();
					}
					let content_type = content_type.unwrap();

					let body_process = Closure::once(Box::new(move |body: JsValue| {
						let body = js_sys::ArrayBuffer::from(body);
						let body =
							js_sys::DataView::new(&body, 0, body.byte_length().try_into().unwrap());

						let mut buffer = vec![];
						for i in 0..body.byte_length() {
							buffer.push(body.get_uint8(i));
						}

						resolve
							.call1(
								&JsValue::NULL,
								&JsValue::from_serde(&Document {
									etag,
									content: buffer,
									content_type,
								})
								.unwrap(),
							)
							.unwrap();
					}) as Box<dyn FnOnce(JsValue)>);

					let body_err = Closure::wrap(Box::new(move |err: JsValue| {
						reject_for_main
							.call1(&JsValue::NULL, &format!("{:?}", err).into())
							.unwrap();
					}) as Box<dyn FnMut(JsValue)>);

					resp.array_buffer()
						.unwrap()
						.then(&body_process)
						.catch(&body_err);

					body_process.forget();
					body_err.forget();
				} else if resp.status() == 404 {
					reject_for_main
						.call1(
							&JsValue::NULL,
							&JsValue::from_str("document does not exists yet in database"),
						)
						.unwrap();
				} else {
					reject_for_main
						.call1(
							&JsValue::NULL,
							&JsValue::from_str(&format!(
								"error {} when access to database",
								resp.status()
							)),
						)
						.unwrap();
				}
			}) as Box<dyn FnOnce(JsValue)>);

			let err_callback = Closure::wrap(Box::new(move |err: JsValue| {
				reject
					.call1(&JsValue::NULL, &format!("{:?}", err).into())
					.unwrap();
			}) as Box<dyn FnMut(JsValue)>);

			window
				.fetch_with_request(&request)
				.then(&process_callback)
				.catch(&err_callback);

			process_callback.forget();
			err_callback.forget();
		}))
	}
	pub fn put_document(
		&self,
		path: impl Into<String>,
		document: &Document,
	) -> Result<Promise, JsValue> {
		let path = path.into();

		let mut opts = web_sys::RequestInit::new();
		opts.method("PUT");
		opts.body(Some(&js_sys::Uint8Array::from(document.content.as_slice())));
		opts.mode(web_sys::RequestMode::Cors);

		let full_path = format!("{}{}", self.server_path, path);

		let request = web_sys::Request::new_with_str_and_init(&full_path, &opts).unwrap();
		request
			.headers()
			.set("Authorization", &format!("Bearer {}", self.access_token))
			.unwrap();
		request
			.headers()
			.set("Content-Type", &document.content_type)
			.unwrap();

		let window = web_sys::window().ok_or("window not found")?;

		Ok(window.fetch_with_request(&request))
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Document {
	etag: Option<String>, // TODO
	content: Vec<u8>,
	content_type: String,
}
impl From<isize> for Document {
	fn from(input: isize) -> Self {
		Self {
			etag: None,
			content: input.to_be_bytes().to_vec(),
			content_type: String::from("text/plain"),
		}
	}
}
impl Document {
	pub fn get_content(&self) -> &[u8] {
		self.content.as_slice()
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct WebfingerResponse {
	links: Vec<Link>,
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct Link {
	href: String,
	properties: std::collections::HashMap<String, Option<String>>,
}
