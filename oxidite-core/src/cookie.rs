use crate::error::{Error, Result};
use crate::extract::FromRequest;
use crate::types::OxiditeRequest;

/// cookie extractor for typed cookie access
///
/// # example
/// ```ignore
/// use oxidite_core::{Cookies, Response};
///
/// async fn handler(cookies: Cookies) -> Result<Response> {
///     if let Some(value) = cookies.get("session_id") {
///         // use cookie value
///     }
///     Ok(Response::ok())
/// }
/// ```
pub struct Cookies {
    cookies: Vec<(String, String)>,
}

impl Cookies {
    /// get cookie value by name
    pub fn get(&self, name: &str) -> Option<&str> {
        self.cookies
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
    
    /// get all cookies
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.cookies.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
    
    /// get mutable access to all cookies
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut String)> {
        self.cookies.iter_mut().map(|(k, v)| (k.as_str(), v))
    }
    
    /// check if cookie exists
    pub fn contains(&self, name: &str) -> bool {
        self.cookies.iter().any(|(k, _)| k == name)
    }
    
    /// get cookie value by name with url decoding
    pub fn get_decoded(&self, name: &str) -> Option<String> {
        self.get(name).map(|value| url_decode(value))
    }
    
    /// get cookie value with validation against malicious content
    pub fn get_safe(&self, name: &str) -> Option<String> {
        self.get(name).map(|value| {
            // basic validation to prevent script injection
            let cleaned = value.replace('<', "&lt;").replace('>', "&gt;");
            cleaned
        })
    }
}

// helper function to decode url-encoded values
fn url_decode(encoded: &str) -> String {
    let mut result = String::new();
    let mut chars = encoded.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            '%' => {
                // try to decode percent-encoded value
                let next_two: String = chars.by_ref().take(2).collect();
                if next_two.len() == 2 {
                    if let Ok(byte_val) = u8::from_str_radix(&next_two, 16) {
                        if let Ok(decoded_char) = std::str::from_utf8(&[byte_val]) {
                            result.push_str(decoded_char);
                        } else {
                            // if decoding fails, keep the original sequence
                            result.push('%');
                            result.push_str(&next_two);
                        }
                    } else {
                        // if parsing hex fails, keep the original sequence
                        result.push('%');
                        result.push_str(&next_two);
                    }
                } else {
                    // if not enough chars, keep the %
                    result.push('%');
                    result.push_str(&next_two);
                }
            },
            '+' => result.push(' '), // commonly used for spaces
            c => result.push(c),
        }
    }
    result
}

/// fromrequest implementation for cookies
impl FromRequest for Cookies {
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self> {
        let cookie_header = req
            .headers()
            .get("cookie")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
            
        let mut cookies = Vec::new();
        
        // parse cookies from header, handling multiple cookies properly
        for cookie_str in cookie_header.split(';') {
            let trimmed = cookie_str.trim();
            
            // skip empty cookies
            if trimmed.is_empty() {
                continue;
            }
            
            // handle cookies with attributes like 'Secure', 'HttpOnly', etc.
            if let Some(pos) = trimmed.find('=') {
                let name = trimmed[..pos].trim();
                let value = trimmed[pos + 1..].trim();
                
                // basic validation to prevent injection
                if !name.is_empty() {
                    // validate cookie name follows standards (alphanumeric, underscore, hyphen)
                    if name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                        cookies.push((name.to_string(), value.to_string()));
                    }
                }
            }
        }
        
        Ok(Cookies { cookies })
    }
}

/// form data extractor for application/x-www-form-urlencoded
///
/// # example
/// ```ignore
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct LoginForm {
///     username: String,
///     password: String,
/// }
///
/// async fn login(form: Form<LoginForm>) -> Result<Response> {
///     // use form.0 to access data
///     Ok(Response::ok())
/// }
/// ```
pub struct Form<T>(pub T);

impl<T: serde::de::DeserializeOwned> FromRequest for Form<T> {
    async fn from_request(req: &mut OxiditeRequest) -> Result<Self> {
        use http_body_util::BodyExt;
        use bytes::Buf;

        let body = req.body_mut();
        let bytes = body.collect().await
            .map_err(|e| Error::Server(format!("Failed to read body: {}", e)))?
            .aggregate();
            
        let mut data = String::new();
        std::io::Read::read_to_string(&mut bytes.reader(), &mut data)
            .map_err(|e| Error::BadRequest(format!("Failed to read body: {}", e)))?;
        
        serde_urlencoded::from_str(&data)
            .map(Form)
            .map_err(|e| Error::BadRequest(format!("Invalid form data: {}", e)))
    }
}
