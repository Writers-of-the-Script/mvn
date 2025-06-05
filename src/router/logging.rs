use axum::{body::Body, http::Request, middleware::Next, response::Response};
use chrono::Utc;

macro_rules! midlog_log {
    ($prefix: expr, $route: expr, $status: expr, $time: expr) => {
        tracing::event!(
            target: "mvn::logging",
            tracing::Level::INFO,
            "{} {} {} {}",
            colored::Colorize::cyan($prefix),
            colored::Colorize::magenta(format!("{}", $route).as_str()),
            $status,
            colored::Colorize::bright_blue(format!("({})", $time).as_str()),
        );
    };
}

pub async fn logging_middleware(req: Request<Body>, next: Next) -> Response {
    debug!("Collecting request info...");

    let time_start = Utc::now().time();
    let method = &req.method().clone();
    let uri = &req.uri().clone();

    debug!("Running route...");

    let res = next.run(req).await;

    debug!("Collecting response info...");

    let now = Utc::now().time();
    let elapsed = now - time_start;
    let path = uri.path_and_query().unwrap().as_str();

    let time = match elapsed.num_microseconds() {
        Some(us) => {
            if us < 1000 {
                format!("{us} µs")
            } else {
                format!("{} ms", us / 1000)
            }
        }

        None => format!("{} ms", elapsed.num_milliseconds()),
    };

    midlog_log!(method.as_str(), path, res.status(), time);

    res
}
