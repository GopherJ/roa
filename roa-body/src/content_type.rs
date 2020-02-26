use crate::help::bug_report;
use mime::{Mime, Name};
use roa_core::http::header::{HeaderValue, CONTENT_TYPE};
use roa_core::http::StatusCode;
use roa_core::{throw, Context, Error, Result, State};
use std::str::FromStr;

fn handle_content_type_error(err: impl ToString) -> Error {
    Error::new(
        StatusCode::BAD_REQUEST,
        format!("{}\nContent-Type value is invalid", err.to_string()),
        true,
    )
}

pub trait Content {
    fn content_type(&self) -> Result<ContentType>;
}

impl<S: State> Content for Context<S> {
    fn content_type(&self) -> Result<ContentType> {
        let req = self.req();
        let value = req.headers.get(CONTENT_TYPE).ok_or_else(|| {
            Error::new(StatusCode::BAD_REQUEST, "Header Content-Type not set", true)
        })?;
        ContentType::form_header_value(value)
    }
}

pub struct ContentType(pub Mime);

impl ContentType {
    pub fn form_header_value(value: &HeaderValue) -> Result<Self> {
        value.to_str().map_err(handle_content_type_error)?.parse()
    }

    pub fn pure_type(&self) -> Result<Mime> {
        let mut ret = format!("{}/{}", self.0.type_(), self.0.subtype());
        if let Some(suffix) = self.0.suffix() {
            ret += "+";
            ret += suffix.as_str();
        }
        ret.parse().map_err(|err| {
            bug_report(format!("{}\n{} is not a valid mime type.", err, ret))
        })
    }

    pub fn charset(&self) -> Option<Name> {
        self.0.get_param("charset")
    }

    pub fn to_value(&self) -> Result<HeaderValue> {
        self.0.as_ref().parse().map_err(bug_report)
    }

    pub fn expect(&self, other: Mime) -> Result {
        let pure_type = self.pure_type()?;
        if pure_type == other {
            Ok(())
        } else {
            throw!(
                StatusCode::BAD_REQUEST,
                format!("content type unmatched. {} != {}", other, pure_type)
            )
        }
    }
}

impl FromStr for ContentType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let mime_type = s.parse().map_err(handle_content_type_error)?;
        Ok(Self(mime_type))
    }
}

#[cfg(test)]
mod test {
    use super::ContentType;
    use test_case::test_case;

    #[test_case("application/json; charset=utf-8; FOO=BAR" => "application/json")]
    #[test_case("image/svg+xml; FOO=BAR" => "image/svg+xml")]
    fn pure_type(mime_type: &str) -> String {
        let content_type: ContentType = mime_type.parse().unwrap();
        content_type.pure_type().unwrap().to_string()
    }
}
