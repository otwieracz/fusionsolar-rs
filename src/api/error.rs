use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use std::io::Cursor;

#[derive(Debug, Clone)]
pub enum Error {
    LoginError(String),
    ApiError(String),
    UnexpectedApiResponse,
    InvalidResponse(String, String),
    UnknownDeviceType(u64),
    RateExceeded(String),
    FormatError,
    InternalError,
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        match self {
            Error::RateExceeded(s) => {
                let error = format!("<html><body><h3>429 Too Many Requests</h3>Downsteram API response: <code>{}</code></body></html>", s);
                Response::build()
                    .status(Status::TooManyRequests)
                    .sized_body(error.len(), Cursor::new(error))
                    .header(ContentType::new("text", "html"))
                    .ok()
            }
            Error::LoginError(s) => {
                let error = format!("<html><body><h3>403 Forbidden</h3>Error white authenticating to downstream API: <code>{}</code></body></html>", s);
                Response::build()
                    .status(Status::Forbidden)
                    .sized_body(error.len(), Cursor::new(error))
                    .header(ContentType::new("text", "html"))
                    .ok()
            }
            _ => {
                let error = format!(
                    "<html><body><h3>Unknown exception</h3><code>{:?}</code></body></html>",
                    self
                );
                Response::build()
                    .status(Status::InternalServerError)
                    .sized_body(error.len(), Cursor::new(error))
                    .header(ContentType::new("text", "html"))
                    .ok()
            }
        }
    }
}
