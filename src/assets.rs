use axum::{
    http::header::{CACHE_CONTROL, CONTENT_TYPE},
    response::Response,
};

const JBM_FONT_BYTES: &[u8] = include_bytes!("../assets/jetbrains-mono.woff2");
const PAGE_JS_CONTENT: &[u8] = include_bytes!("../assets/page.js");
const ROBOTS_TXT_BYTES: &[u8] = include_bytes!("../assets/robots.txt");

#[axum::debug_handler]
pub async fn jbm_font_route() -> Response {
    Response::builder()
        .status(200)
        .header(CONTENT_TYPE, "font/woff2")
        .header(CACHE_CONTROL, "max-age=31536000")
        .body(JBM_FONT_BYTES.into())
        .unwrap()
}

#[axum::debug_handler]
pub async fn page_js_route() -> Response {
    Response::builder()
        .status(200)
        .header(CONTENT_TYPE, "text/javascript")
        .body(PAGE_JS_CONTENT.into())
        .unwrap()
}

#[axum::debug_handler]
pub async fn robots_txt_route() -> Response {
    Response::builder()
        .status(200)
        .header(CONTENT_TYPE, "text/plain")
        .body(ROBOTS_TXT_BYTES.into())
        .unwrap()
}
