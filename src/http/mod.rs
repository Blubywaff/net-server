use std::{
    net::TcpStream,
    collections::HashMap,
    str,
};

const CR: u8 = 0x0du8;
const SP: u8 = 0x20u8;

pub struct HttpRequest {
    stream: Option<TcpStream>,
    method: String,
    path: String,
    query: HashMap<String, Vec<String>>,
    protocol: String,
    headers: HashMap<String, Vec<u8>>,
}

impl HttpRequest {
    pub fn empty() -> Self {
        HttpRequest {
            stream: None,
            method: String::new(),
            path: String::new(),
            query: HashMap::new(),
            protocol: String::new(),
            headers: HashMap::new(),
        }
    }
    /// Expects headers and status line to be at most 8192 bytes long
    pub fn from_stream(stream: TcpStream) -> Result<Self, HttpReadError> {
        let rq = Self {
            stream: Some(stream),
            ..Self::empty()
        };
        let tcp_buf: [u8; 8192] = [0u8; 8192];
        let len = match rq.stream.unwrap().peek(&mut tcp_buf) {
            Ok(l) => l,
            Err(_) => return Err(HttpReadError {})
        };
        let mut c: usize = 0;
        { // method
            rq.method = match String::from_utf8(
                match slice_until(&tcp_buf[c..], &SP) {
                    Some(b) => {
                        c += b.len() + 1;
                        b.to_vec()
                    },
                    None => return Err(HttpReadError {})
                }
            ) {
                Ok(s) => s,
                Err(_) => return Err(HttpReadError {})
            };
        };
        { // path and query
            let ptmp = match String::from_utf8(
                match slice_until(&tcp_buf[c..], &SP) {
                    Some(b) => {
                        c += b.len() + 1;
                        b.to_vec()
                    },
                    None => return Err(HttpReadError {})
                }
            ) {
                Ok(s) => s,
                Err(_) => return Err(HttpReadError {})
            };
            let (path, query) = match ptmp.split_once("?") {
                Some((p, q)) => (p.to_string(), q),
                None => (ptmp, "")
            };
            rq.path = path;
            for q in query.split("&") {
                if let (k, v) = q.split_once("=").unwrap() {
                    let k = k.to_string();
                    let v = v.to_string();
                    if rq.query.contains_key(&k) {
                        rq.query.get_mut(&k).unwrap().push(v);
                    } else {
                        rq.query.insert(k, vec![v]);
                    }
                } else {
                    return Err(HttpReadError {})
                }
            }
        }
        { // protocol
            rq.protocol = match String::from_utf8(
                match slice_until(&tcp_buf[c..], &CR) {
                    Some(b) => {
                        c += b.len() + 2;
                        b.to_vec()
                    },
                    None => return Err(HttpReadError {})
                }
            ) {
                Ok(s) => s,
                Err(_) => return Err(HttpReadError {})
            };
        }
        { // headers
            'outer: loop {
                let name = match String::from_utf8(
                    match slice_to_or_exit(&tcp_buf[c..], &b':', &CR) {
                        (l, Some(b)) =>  {
                            c += l;
                            b.to_vec()
                        },
                        (l, None) => break 'outer
                    }
                ) {
                    Ok(s) => s,
                    Err(_) => return Err(HttpReadError {})
                };
                let value = match slice_until(&tcp_buf[c..], &CR) {
                    Some(b) => b.to_vec(),
                    None => return Err(HttpReadError {})
                };
                match rq.headers.get_mut(&name) {
                    Some(v) => {
                        v.append(&mut value)
                    },
                    None => match rq.headers.insert(name, value) {
                        // Covered by `get_mut`, only get to this branch
                        // if key does not exist, so it should also not
                        // have a value.
                        Some(_) => unreachable!(),
                        None => {},
                    }
                }
                if rq.headers.contains_key(&name) {
                } else {
                    rq.headers.insert(name, value);
                }
            }
        }
        return Ok(rq);
    }
}

/// Reads bytes from `buf` until encountering `b`
/// and returns the byte slice from the beginning of `buf`
/// up to and excluding `b`
///
/// Returns None if `b` is not encountered
fn slice_until<'a>(buf: &'a [u8], b: &u8) -> Option<&'a [u8]> {
    for (i, x) in buf.iter().enumerate() {
        if x == b {
            return Some(&buf[..i])
        }
    }
    return None
}

/// Reads bytes from `buf` until encountering `b` or `e`
/// returns the number of bytes read until exit.
/// Returns (usize, None), if `b` is not encountered.
/// Returns (usize, Some), if `b` is encountered.
fn slice_to_or_exit<'a>(buf: &'a [u8], b: &u8, e:&u8) -> (usize, Option<&'a [u8]>) {
    let mut i: usize = 0;
    for (i, x) in buf.iter().enumerate() {
        if x == b {
            return (i, Some(&buf[..i]))
        }
    }
    return (i, None)
}

pub struct HttpReadError {
}
