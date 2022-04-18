use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

lazy_static::lazy_static! {
	static ref ACCESS_TOKEN_REGEX: regex::Regex = regex::Regex::new("^#.*access_token=([^&]+).+$").unwrap();
}

const OAUTH_KEY: &str = "http://tools.ietf.org/html/rfc6749#section-4.2";

pub struct Client {
	webfinger_root_uri: String,
	username: String,
	scope: String,
	client_id: String,
	server_path: String,
	access_token: String,
	debug: bool, // TODO
}
impl Client {
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

		let window = web_sys::window().ok_or(JsValue::from_str("window not found"))?;
		let document = window
			.document()
			.ok_or(JsValue::from_str("document not found"))?;
		let document = document.dyn_ref::<web_sys::HtmlDocument>().unwrap();

		let webfinger_root_uri_obj = webfinger_root_uri.parse::<http::uri::Uri>().unwrap();
		let client_id_uri_obj = client_id.parse::<http::uri::Uri>().unwrap();

		let cookie_name_header = format!(
			"{}|{username}|{}|{scope}|",
			match client_id_uri_obj.port() {
				Some(port) => format!("{}:{}", client_id_uri_obj.host().unwrap(), port),
				None => String::from(client_id_uri_obj.host().unwrap()),
			},
			match webfinger_root_uri_obj.port() {
				Some(port) => format!("{}:{}", webfinger_root_uri_obj.host().unwrap(), port),
				None => String::from(webfinger_root_uri_obj.host().unwrap()),
			}
		);
		let cookie_name_header =
			pct_str::PctString::encode(cookie_name_header.chars(), pct_str::URIReserved);

		////////////////////////

		let mut opts = web_sys::RequestInit::new();
		opts.method("GET");
		opts.mode(web_sys::RequestMode::Cors);

		let webfinger_uri = webfinger_root_uri
			.strip_suffix('/')
			.unwrap_or(&webfinger_root_uri);
		let url = format!(
			"{webfinger_uri}/.well-known/webfinger?resource=acct:{username}@{}",
			webfinger_root_uri_obj.host().unwrap()
		);

		let request = web_sys::Request::new_with_str_and_init(&url, &opts)?;

		let resp_value =
			wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
		let resp: web_sys::Response = resp_value.dyn_into()?;
		let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;
		let response: WebfingerResponse = json.into_serde().unwrap();

		let link = response.links[0].clone();
		let server_path = link.href;

		////////////////////////

		let all_cookies = document.cookie()?;

		let mut access_token = None;
		for cookie in all_cookies.split(';') {
			let mut iter = cookie.split('=');
			let name = iter.next().map(str::trim);
			let value = iter
				.next()
				.map(|res| pct_str::PctString::new(res.trim()).unwrap().decode());

			if let Some(name) = name {
				if name == &format!("{cookie_name_header}access_token") {
					access_token = value;
				}
			}
		}

		let access_token = match access_token {
			Some(access_token) => Some(access_token),
			None => {
				let hash = window.location().hash();

				access_token = match hash {
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
							let oauth_origin =
								link.properties.get(OAUTH_KEY).unwrap().as_ref().unwrap();
							let oauth_path = format!(
								"{oauth_origin}?redirect_uri={}&scope={}&client_id={}&response_type={}",
								pct_str::PctString::encode(
									format!("{}", window.location().to_string()).chars(),
									pct_str::URIReserved
								), // TODO : change to base url (no page name, or its arguments)
								pct_str::PctString::encode(scope.chars(), pct_str::URIReserved),
								pct_str::PctString::encode(client_id.chars(), pct_str::URIReserved),
								pct_str::PctString::encode("token".chars(), pct_str::URIReserved),
							);

							// window.location().set_href(&oauth_path).unwrap();

							let next_window = document.create_element("div")?;
							next_window.set_attribute("id", "pontus_onyx_oauth_next_window");
							let next_window =
								next_window.dyn_ref::<web_sys::HtmlElement>().unwrap();

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
							next_window.append_child(&svg);

							let explain = document.create_element("p")?;
							explain.set_inner_html(
								&format!(
									r#"You will be temporary redirected to<br><a href="{}">{}</a><br>in order to authenticate on the requested remoteStorage server, then bring back to this page."#,
									oauth_path,
									oauth_origin
								)
							);

							next_window.append_child(&explain);

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
							let close_next_window =
								wasm_bindgen::closure::Closure::wrap(Box::new(move || {
									if let Some(window) = web_sys::window() {
										if let Some(document) = window.document() {
											if let Some(body) = document.body() {
												if let Some(node) = document.get_element_by_id(
													"pontus_onyx_oauth_next_window",
												) {
													body.remove_child(&node).ok();
												}
											}
										}
									}
								}) as Box<dyn FnMut()>);
							abort.set_onclick(Some(close_next_window.as_ref().unchecked_ref()));
							close_next_window.forget();

							p_buttons.append_child(&abort);

							let a_next = document.create_element("a")?;
							a_next.set_attribute("href", &oauth_path);

							let button_next = document.create_element("button")?;
							let button_next =
								button_next.dyn_ref::<web_sys::HtmlElement>().unwrap();
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
							a_next.append_child(&button_next);

							p_buttons.append_child(&a_next);

							next_window.append_child(&p_buttons);

							document.body().unwrap().append_child(&next_window);

							// TODO : automatic redirection ?

							None
						}
					}
					Err(_) => None,
				};

				match access_token {
					Some(access_token) => {
						// hide token from URL
						window.history()?.replace_state_with_url(
							&String::new().into(),
							"",
							Some("/"),
						)?;

						document
							.set_cookie(&format!(
								"{}={}",
								format!("{cookie_name_header}access_token"),
								pct_str::PctString::encode(
									access_token.chars(),
									pct_str::URIReserved
								)
							))
							.unwrap();

						Some(access_token)
					}
					None => None,
				}
			}
		};

		////////////////////////

		match access_token {
			Some(access_token) => {
				Ok(Self {
					webfinger_root_uri,
					username,
					scope,
					client_id,
					server_path,
					access_token, // TODO
					debug,        // TODO
				})
			}
			None => Err(JsValue::from_str("can not obtain access token from server")),
		}
	}
}
impl Client {
	pub fn get_document(
		&self,
		path: impl Into<String>,
		etag: Option<String>, // TODO
	) -> Result<js_sys::Promise, JsValue> {
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

		Ok(window.fetch_with_request(&request))
	}
	pub fn put_document(
		&self,
		path: impl Into<String>,
		document: &Document,
	) -> Result<js_sys::Promise, JsValue> {
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

#[derive(Debug)]
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct WebfingerResponse {
	links: Vec<Link>,
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct Link {
	href: String,
	properties: std::collections::HashMap<String, Option<String>>,
}
