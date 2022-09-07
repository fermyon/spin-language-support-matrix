use anyhow::Result;
use clap::{Parser, Subcommand};
use spin_http::{Request, Response};
use std::{error, fmt};

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

wit_bindgen_rust::import!("../../wit/spin-config.wit");

impl fmt::Display for spin_config::Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Provider(provider_err) => write!(f, "provider error: {}", provider_err),
            Self::InvalidKey(invalid_key) => write!(f, "invalid key: {}", invalid_key),
            Self::InvalidSchema(invalid_schema) => {
                write!(f, "invalid schema: {}", invalid_schema)
            }
            Self::Other(other) => write!(f, "other: {}", other),
        }
    }
}

impl error::Error for spin_config::Error {}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Config { key: String },
}

fn main() -> Result<()> {
    match &Cli::parse().command {
        Command::Config { key } => {
            print!("{}", spin_config::get_config(key)?);
        }
    }

    Ok(())
}
