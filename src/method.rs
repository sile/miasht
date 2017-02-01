use std::fmt;

/// HTTP Method.
///
/// See [IANA's HTTP Method Registry]
/// (https://www.iana.org/assignments/http-methods/http-methods.xhtml)
/// for more details about each method.
///
/// # Examples
///
/// ```
/// use miasht::Method;
///
/// assert_eq!(Method::try_from_str("GET"), Some(Method::Get));
/// assert_eq!(Method::try_from_str("get"), None); // case senstive
///
/// let method = Method::try_from_str("GET").unwrap();
/// assert_eq!(method.as_str(), "GET");
/// assert_eq!(method.to_string(), "GET");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Acl,
    BaselineControl,
    Bind,
    Checkin,
    Checkout,
    Connect,
    Copy,
    Delete,
    Get,
    Head,
    Label,
    Link,
    Lock,
    Merge,
    Mkactivity,
    Mkcalendar,
    Mkcol,
    Mkredirectref,
    Mkworkspace,
    Move,
    Options,
    Orderpatch,
    Patch,
    Post,
    Pri,
    Propfind,
    Proppatch,
    Put,
    Rebind,
    Report,
    Search,
    Trace,
    Unbind,
    Uncheckout,
    Unlink,
    Unlock,
    Update,
    Updateredirectref,
    VersionControl,
}
impl Method {
    pub fn try_from_str(method: &str) -> Option<Self> {
        Some(match method {
            "ACL" => Method::Acl,
            "BASELINE-CONTROL" => Method::BaselineControl,
            "BIND" => Method::Bind,
            "CHECKIN" => Method::Checkin,
            "CHECKOUT" => Method::Checkout,
            "CONNECT" => Method::Connect,
            "COPY" => Method::Copy,
            "DELETE" => Method::Delete,
            "GET" => Method::Get,
            "HEAD" => Method::Head,
            "LABEL" => Method::Label,
            "LINK" => Method::Link,
            "LOCK" => Method::Lock,
            "MERGE" => Method::Merge,
            "MKACTIVITY" => Method::Mkactivity,
            "MKCALENDAR" => Method::Mkcalendar,
            "MKCOL" => Method::Mkcol,
            "MKREDIRECTREF" => Method::Mkredirectref,
            "MKWORKSPACE" => Method::Mkworkspace,
            "MOVE" => Method::Move,
            "OPTIONS" => Method::Options,
            "ORDERPATCH" => Method::Orderpatch,
            "PATCH" => Method::Patch,
            "POST" => Method::Post,
            "PRI" => Method::Pri,
            "PROPFIND" => Method::Propfind,
            "PROPPATCH" => Method::Proppatch,
            "PUT" => Method::Put,
            "REBIND" => Method::Rebind,
            "REPORT" => Method::Report,
            "SEARCH" => Method::Search,
            "TRACE" => Method::Trace,
            "UNBIND" => Method::Unbind,
            "UNCHECKOUT" => Method::Uncheckout,
            "UNLINK" => Method::Unlink,
            "UNLOCK" => Method::Unlock,
            "UPDATE" => Method::Update,
            "UPDATEREDIRECTREF" => Method::Updateredirectref,
            "VERSION-CONTROL" => Method::VersionControl,
            _ => return None,
        })
    }
    pub fn as_str(&self) -> &str {
        match *self {
            Method::Acl => "ACL",
            Method::BaselineControl => "BASELINE-CONTROL",
            Method::Bind => "BIND",
            Method::Checkin => "CHECKIN",
            Method::Checkout => "CHECKOUT",
            Method::Connect => "CONNECT",
            Method::Copy => "COPY",
            Method::Delete => "DELETE",
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Label => "LABEL",
            Method::Link => "LINK",
            Method::Lock => "LOCK",
            Method::Merge => "MERGE",
            Method::Mkactivity => "MKACTIVITY",
            Method::Mkcalendar => "MKCALENDAR",
            Method::Mkcol => "MKCOL",
            Method::Mkredirectref => "MKREDIRECTREF",
            Method::Mkworkspace => "MKWORKSPACE",
            Method::Move => "MOVE",
            Method::Options => "OPTIONS",
            Method::Orderpatch => "ORDERPATCH",
            Method::Patch => "PATCH",
            Method::Post => "POST",
            Method::Pri => "PRI",
            Method::Propfind => "PROPFIND",
            Method::Proppatch => "PROPPATCH",
            Method::Put => "PUT",
            Method::Rebind => "REBIND",
            Method::Report => "REPORT",
            Method::Search => "SEARCH",
            Method::Trace => "TRACE",
            Method::Unbind => "UNBIND",
            Method::Uncheckout => "UNCHECKOUT",
            Method::Unlink => "UNLINK",
            Method::Unlock => "UNLOCK",
            Method::Update => "UPDATE",
            Method::Updateredirectref => "UPDATEREDIRECTREF",
            Method::VersionControl => "VERSION-CONTROL",
        }
    }
}
impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
