use std::fmt;

#[derive(Debug, Clone)]
pub struct RawStatus<'a> {
    code: u16,
    reason: &'a str,
}
impl<'a> RawStatus<'a> {
    pub fn new(code: u16, reason: &'a str) -> Self {
        RawStatus {
            code: code,
            reason: reason,
        }
    }
    pub fn code(&self) -> u16 {
        self.code
    }
    pub fn reason(&self) -> &str {
        self.reason
    }
    pub fn normalize(&self) -> Option<Status> {
        Some(match self.code {
            100 => Status::Continue,
            101 => Status::SwitchingProtocols,
            102 => Status::Processing,

            200 => Status::Ok,
            201 => Status::Created,
            202 => Status::Accepted,
            203 => Status::NonAuthoritativeInformation,
            204 => Status::NoContent,
            205 => Status::ResetContent,
            206 => Status::PartialContent,
            207 => Status::MultiStatus,
            208 => Status::AlreadyReported,
            226 => Status::ImUsed,

            300 => Status::MultipleChoices,
            301 => Status::MovedPermanently,
            302 => Status::Found,
            303 => Status::SeeOther,
            304 => Status::NotModified,
            305 => Status::UseProxy,
            307 => Status::TemporaryRedirect,
            308 => Status::PermanentRedirect,

            400 => Status::BadRequest,
            401 => Status::Unauthorized,
            402 => Status::PaymentRequired,
            403 => Status::Forbidden,
            404 => Status::NotFound,
            405 => Status::MethodNotAllowed,
            406 => Status::NotAcceptable,
            407 => Status::ProxyAuthenticationRequired,
            408 => Status::RequestTimeout,
            409 => Status::Conflict,
            410 => Status::Gone,
            411 => Status::LengthRequired,
            412 => Status::PreconditionFailed,
            413 => Status::PayloadTooLarge,
            414 => Status::UriTooLong,
            415 => Status::UnsupportedMediaType,
            416 => Status::RangeNotSatisfiable,
            417 => Status::ExceptionFailed,
            418 => Status::ImATeapot,
            421 => Status::MisdirectedRequest,
            422 => Status::UnprocessableEntity,
            423 => Status::Locked,
            424 => Status::FailedDependency,
            426 => Status::UpgradeRequired,
            451 => Status::UnavailableForLegalReasons,

            500 => Status::InternalServerError,
            501 => Status::NotImplemented,
            502 => Status::BadGateway,
            503 => Status::ServiceUnavailable,
            504 => Status::GatewayTimeout,
            505 => Status::HttpVersionNotSupported,
            506 => Status::VariantAlsoNegotiates,
            507 => Status::InsufficientStorage,
            508 => Status::LoopDetected,
            509 => Status::BandwidthLimitExceeded,
            510 => Status::NotExtended,
            _ => return None,
        })
    }
}
impl<'a> fmt::Display for RawStatus<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.code, self.reason)
    }
}
impl From<Status> for RawStatus<'static> {
    fn from(f: Status) -> Self {
        f.as_raw()
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Status {
    // 1xxx
    Continue,
    SwitchingProtocols,
    Processing,

    // 2xx
    Ok,
    Created,
    Accepted,
    NonAuthoritativeInformation,
    NoContent,
    ResetContent,
    PartialContent,
    MultiStatus,
    AlreadyReported,
    ImUsed,

    // 3xx
    MultipleChoices,
    MovedPermanently,
    Found,
    SeeOther,
    NotModified,
    UseProxy,
    TemporaryRedirect,
    PermanentRedirect,

    // 4xx
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    ProxyAuthenticationRequired,
    RequestTimeout,
    Conflict,
    Gone,
    LengthRequired,
    PreconditionFailed,
    PayloadTooLarge,
    UriTooLong,
    UnsupportedMediaType,
    RangeNotSatisfiable,
    ExceptionFailed,
    ImATeapot,
    MisdirectedRequest,
    UnprocessableEntity,
    Locked,
    FailedDependency,
    UpgradeRequired,
    UnavailableForLegalReasons,

    // 5xx
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HttpVersionNotSupported,
    VariantAlsoNegotiates,
    InsufficientStorage,
    LoopDetected,
    BandwidthLimitExceeded,
    NotExtended,
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.code(), self.reason_phrase())
    }
}
impl Status {
    pub fn as_raw(&self) -> RawStatus<'static> {
        RawStatus::new(self.code(), self.reason_phrase())
    }
    pub fn code(&self) -> u16 {
        match *self {
            Status::Continue => 100,
            Status::SwitchingProtocols => 101,
            Status::Processing => 102,

            Status::Ok => 200,
            Status::Created => 201,
            Status::Accepted => 202,
            Status::NonAuthoritativeInformation => 203,
            Status::NoContent => 204,
            Status::ResetContent => 205,
            Status::PartialContent => 206,
            Status::MultiStatus => 207,
            Status::AlreadyReported => 208,
            Status::ImUsed => 226,

            Status::MultipleChoices => 300,
            Status::MovedPermanently => 301,
            Status::Found => 302,
            Status::SeeOther => 303,
            Status::NotModified => 304,
            Status::UseProxy => 305,
            Status::TemporaryRedirect => 307,
            Status::PermanentRedirect => 308,

            Status::BadRequest => 400,
            Status::Unauthorized => 401,
            Status::PaymentRequired => 402,
            Status::Forbidden => 403,
            Status::NotFound => 404,
            Status::MethodNotAllowed => 405,
            Status::NotAcceptable => 406,
            Status::ProxyAuthenticationRequired => 407,
            Status::RequestTimeout => 408,
            Status::Conflict => 409,
            Status::Gone => 410,
            Status::LengthRequired => 411,
            Status::PreconditionFailed => 412,
            Status::PayloadTooLarge => 413,
            Status::UriTooLong => 414,
            Status::UnsupportedMediaType => 415,
            Status::RangeNotSatisfiable => 416,
            Status::ExceptionFailed => 417,
            Status::ImATeapot => 418,
            Status::MisdirectedRequest => 421,
            Status::UnprocessableEntity => 422,
            Status::Locked => 423,
            Status::FailedDependency => 424,
            Status::UpgradeRequired => 426,
            Status::UnavailableForLegalReasons => 451,

            Status::InternalServerError => 500,
            Status::NotImplemented => 501,
            Status::BadGateway => 502,
            Status::ServiceUnavailable => 503,
            Status::GatewayTimeout => 504,
            Status::HttpVersionNotSupported => 505,
            Status::VariantAlsoNegotiates => 506,
            Status::InsufficientStorage => 507,
            Status::LoopDetected => 508,
            Status::BandwidthLimitExceeded => 509,
            Status::NotExtended => 510,
        }
    }
    pub fn reason_phrase(&self) -> &'static str {
        match *self {
            Status::Continue => "Continue",
            Status::SwitchingProtocols => "Switching Protocols",
            Status::Processing => "Processing",

            Status::Ok => "OK",
            Status::Created => "Created",
            Status::Accepted => "Accepted",
            Status::NonAuthoritativeInformation => "Non-Authoritative Information",
            Status::NoContent => "No Content",
            Status::ResetContent => "Reset Content",
            Status::PartialContent => "Partial Content",
            Status::MultiStatus => "Multi-Status",
            Status::AlreadyReported => "Already Reported",
            Status::ImUsed => "IM Used",

            Status::MultipleChoices => "Multiple Choices",
            Status::MovedPermanently => "Moved Permanently",
            Status::Found => "Found",
            Status::SeeOther => "See Other",
            Status::NotModified => "Not Modified",
            Status::UseProxy => "Use Proxy",
            Status::TemporaryRedirect => "Temporary Redirect",
            Status::PermanentRedirect => "Permanent Redirect",

            Status::BadRequest => "Bad Request",
            Status::Unauthorized => "Unauthorized",
            Status::PaymentRequired => "Payment Required",
            Status::Forbidden => "Forbidden",
            Status::NotFound => "Not Found",
            Status::MethodNotAllowed => "Method Not Allowed",
            Status::NotAcceptable => "Not Acceptable",
            Status::ProxyAuthenticationRequired => "Proxy Authentication Required",
            Status::RequestTimeout => "Request Timeout",
            Status::Conflict => "Conflict",
            Status::Gone => "Gone",
            Status::LengthRequired => "Length Required",
            Status::PreconditionFailed => "Precondition Failed",
            Status::PayloadTooLarge => "Payload Too Large",
            Status::UriTooLong => "URI Too Long",
            Status::UnsupportedMediaType => "Unsupported Media Type",
            Status::RangeNotSatisfiable => "Range Not Satisfiable",
            Status::ExceptionFailed => "Expectation Failed",
            Status::ImATeapot => "I'm a teapot",
            Status::MisdirectedRequest => "Misdirected Request",
            Status::UnprocessableEntity => "Unporcessable Entity",
            Status::Locked => "Locked",
            Status::FailedDependency => "Failed Dependency",
            Status::UpgradeRequired => "Upgrade Required",
            Status::UnavailableForLegalReasons => "Unavailable For Legal Reasons",

            Status::InternalServerError => "Internal Server Error",
            Status::NotImplemented => "Not Implemented",
            Status::BadGateway => "Bad Gateway",
            Status::ServiceUnavailable => "Service Unavailable",
            Status::GatewayTimeout => "Gateway Timeout",
            Status::HttpVersionNotSupported => "HTTP Version Not Supported",
            Status::VariantAlsoNegotiates => "Variant Also Negotiates",
            Status::InsufficientStorage => "Insufficient Storage",
            Status::LoopDetected => "Loop Detected",
            Status::BandwidthLimitExceeded => "Bandwidth Limit Exceeded",
            Status::NotExtended => "Not Extended",
        }
    }
}
