use anyhow::{anyhow, bail, Result};
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
            Self::Provider(provider_err) => write!(f, "provider error: {provider_err}"),
            Self::InvalidKey(invalid_key) => write!(f, "invalid key: {invalid_key}"),
            Self::InvalidSchema(invalid_schema) => write!(f, "invalid schema: {invalid_schema}"),
            Self::Other(other) => write!(f, "other: {other}"),
        }
    }
}

impl error::Error for spin_config::Error {}

wit_bindgen_rust::import!("../../wit/wasi-outbound-http.wit");

impl fmt::Display for wasi_outbound_http::HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Success => "success",
            Self::DestinationNotAllowed => "destination not allowed",
            Self::InvalidUrl => "invalid url",
            Self::RequestError => "request error",
            Self::RuntimeError => "runtime error",
            Self::TooManyRequests => "too many requests",
        })
    }
}

impl error::Error for wasi_outbound_http::HttpError {}

wit_bindgen_rust::import!("../../wit/outbound-redis.wit");

impl fmt::Display for outbound_redis::Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Success => "success",
            Self::Error => "error",
        })
    }
}

impl error::Error for outbound_redis::Error {}

wit_bindgen_rust::import!("../../wit/outbound-pg.wit");

impl fmt::Display for outbound_pg::PgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Success => f.write_str("success"),
            Self::ConnectionFailed(message) => write!(f, "connection failed: {message}"),
            Self::BadParameter(message) => write!(f, "bad parameter: {message}"),
            Self::QueryFailed(message) => write!(f, "query failed: {message}"),
            Self::ValueConversionFailed(message) => write!(f, "value conversion failed: {message}"),
            Self::OtherError(message) => write!(f, "error: {message}"),
        }
    }
}

impl error::Error for outbound_pg::PgError {}

fn parse(param: &str) -> Result<outbound_pg::ParameterValue> {
    use outbound_pg::ParameterValue as PV;

    Ok(if param == "null" {
        PV::DbNull
    } else {
        let (type_, value) = param
            .split_once(':')
            .ok_or_else(|| anyhow!("expected ':' in {param}"))?;

        match type_ {
            "boolean" => PV::Boolean(value.parse()?),
            "int8" => PV::Int8(value.parse()?),
            "int16" => PV::Int16(value.parse()?),
            "int32" => PV::Int32(value.parse()?),
            "int64" => PV::Int64(value.parse()?),
            "uint8" => PV::Uint8(value.parse()?),
            "uint16" => PV::Uint16(value.parse()?),
            "uint32" => PV::Uint32(value.parse()?),
            "uint64" => PV::Uint64(value.parse()?),
            "floating32" => PV::Floating32(value.parse()?),
            "floating64" => PV::Floating64(value.parse()?),
            "str" => PV::Str(value),
            "binary" => PV::Binary(value.as_bytes()),
            _ => bail!("unknown parameter type: {type_}"),
        }
    })
}

impl fmt::Display for outbound_pg::DbValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Int8(value) => write!(f, "{value}"),
            Self::Int16(value) => write!(f, "{value}"),
            Self::Int32(value) => write!(f, "{value}"),
            Self::Int64(value) => write!(f, "{value}"),
            Self::Uint8(value) => write!(f, "{value}"),
            Self::Uint16(value) => write!(f, "{value}"),
            Self::Uint32(value) => write!(f, "{value}"),
            Self::Uint64(value) => write!(f, "{value}"),
            Self::Floating32(value) => write!(f, "{value}"),
            Self::Floating64(value) => write!(f, "{value}"),
            Self::Str(value) => write!(f, "{value}"),
            Self::Binary(value) => write!(f, "{value:?}"),
            Self::DbNull => Ok(()),
            Self::Unsupported => write!(f, "<unsupported>"),
        }
    }
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Config {
        key: String,
    },
    OutboundHttp {
        url: String,
    },
    OutboundRedisPublish {
        address: String,
        key: String,
        value: String,
    },
    OutboundRedisSet {
        address: String,
        key: String,
        value: String,
    },
    OutboundRedisGet {
        address: String,
        key: String,
    },
    OutboundRedisIncr {
        address: String,
        key: String,
    },
    OutboundPgExecute {
        address: String,
        statement: String,
        params: Vec<String>,
    },
    OutboundPgQuery {
        address: String,
        statement: String,
        params: Vec<String>,
    },
}

fn main() -> Result<()> {
    match &Cli::parse().command {
        Command::Config { key } => {
            print!("{}", spin_config::get_config(key)?);
        }

        Command::OutboundHttp { url } => {
            use wasi_outbound_http::{Method, Request};

            print!(
                "{}",
                wasi_outbound_http::request(Request {
                    method: Method::Get,
                    uri: url,
                    headers: &[],
                    params: &[],
                    body: None
                })?
                .body
                .and_then(|body| String::from_utf8(body).ok())
                .unwrap_or_else(String::new)
            )
        }

        Command::OutboundRedisPublish {
            address,
            key,
            value,
        } => {
            outbound_redis::publish(address, key, value.as_bytes())?;

            print!("success");
        }

        Command::OutboundRedisSet {
            address,
            key,
            value,
        } => {
            outbound_redis::set(address, key, value.as_bytes())?;

            print!("success");
        }

        Command::OutboundRedisGet { address, key } => {
            print!("{}", String::from_utf8(outbound_redis::get(address, key)?)?);
        }

        Command::OutboundRedisIncr { address, key } => {
            print!("{}", outbound_redis::incr(address, key)?);
        }

        Command::OutboundPgExecute {
            address,
            statement,
            params,
        } => {
            print!(
                "{}",
                outbound_pg::execute(
                    address,
                    statement,
                    &params
                        .iter()
                        .map(|param| parse(param))
                        .collect::<Result<Vec<_>>>()?
                )?
            );
        }

        Command::OutboundPgQuery {
            address,
            statement,
            params,
        } => {
            let row_set = outbound_pg::query(
                address,
                statement,
                &params
                    .iter()
                    .map(|param| parse(param))
                    .collect::<Result<Vec<_>>>()?,
            )?;

            let mut newline = false;
            for row in &row_set.rows {
                if newline {
                    println!();
                } else {
                    newline = true;
                }

                let mut comma = false;
                for value in row {
                    if comma {
                        print!(",")
                    } else {
                        comma = true;
                    }
                    print!("{value}",);
                }
            }
        }
    }

    Ok(())
}
