mod request;
mod response;
mod status;

use std::{fmt, str::FromStr};

pub use request::HttpRequest;
pub use response::HttpResponse;
pub use status::Status;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Method {
    Connect,
    Delete,
    #[default]
    Get,
    Head,
    Options,
    Post,
    Put,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect => write!(f, "CONNECT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Get => write!(f, "GET"),
            Self::Head => write!(f, "HEAD"),
            Self::Options => write!(f, "OPTIONS"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
        }
    }
}

impl FromStr for Method {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CONNECT" => Ok(Self::Connect),
            "DELETE" => Ok(Self::Delete),
            "GET" => Ok(Self::Get),
            "HEAD" => Ok(Self::Head),
            "OPTIONS" => Ok(Self::Options),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            _ => Err(()),
        }
    }
}
