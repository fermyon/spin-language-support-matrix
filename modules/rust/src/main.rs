use spin_http::{Request, Response};

wit_bindgen_rust::export!("../../wit/spin-http.wit");

struct SpinHttp;

impl spin_http::SpinHttp for SpinHttp {
    fn handle_http_request(request: Request) -> Response {
        Response {
            status: 200,
            headers: Some(request.headers),
            body: request
                .body
                .map(|body| b"you said: ".iter().copied().chain(body).collect()),
        }
    }
}

wit_bindgen_rust::export!("../../wit/spin-redis.wit");

struct SpinRedis;

impl spin_redis::SpinRedis for SpinRedis {
    fn handle_redis_message(_body: Vec<u8>) -> Result<(), spin_redis::Error> {
        Ok(())
    }
}

fn main() {
    println!("Hello, world!");
}
