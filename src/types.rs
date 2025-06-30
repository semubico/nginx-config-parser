use std::{net::SocketAddr, path::PathBuf, str::FromStr, time::Duration};
use regex::Regex;
use url::Url;

use crate::Structure;

pub enum Location {
    Exact(String),
    Prefix(String),
    Incasitive(Regex),
    Casitive(Regex),
    Virtual(String),
}

impl Location {
    
    pub fn matches(&self, path: &str) -> bool {
        match self {
            Self::Exact(exact) => path.eq_ignore_ascii_case(exact),
            Self::Prefix(prefix) => path.starts_with(prefix),
            Self::Incasitive(pattern) => pattern.is_match(path),    // TODO
            Self::Casitive(pattern) => pattern.is_match(path),            
            _ => false
        }
    }

    pub fn matches_internal(&self, path: &str) -> bool {
        match self {
            Self::Virtual(virt) => path.eq_ignore_ascii_case(virt),
            _ => false
        }
    }    
}

pub enum Directive {
    ErrorLog {
        path: PathBuf,
        level: Option<String>
    },
    AccessLog {
        path: PathBuf,
        compression_enabled: bool
    },
    AddHeader{
        name: String,
        value: String
    },
    AuthBasic {
        realm: String
    },
    AuthBasicUserFile {
        file: PathBuf  
    },
    Http2 {
        enabled: bool
    },
    Listen {
        sock_addr: SocketAddr,
        is_default: bool,
        is_http2: bool,
        is_http3: bool
    },
    ProxyHttpVersion {
        version: String
    },
    ProxyPass {
        addr: Url
    },
    ProxyReadTimeout {
        timeout: Duration
    },
    ProxySetHeader {
        header_name: String,
        header_value: String
    },
    ProxyHideHeader {
        header_name: String  
    },
    Return {
        code: Option<u16>,
        content: Option<String>
    },
    ServerName {
        name: String
    },   
    ServerTokens {
        enabled: bool
    },
    SslCertificate {
        path: PathBuf
    },
    SslCertificateKey {
        path: PathBuf
    },
    SslEarlyData {
        enabled: bool
    },
    Location(Location),
}

impl<'l> TryFrom<crate::Structure<'l>> for Directive {
    type Error = ();
    fn try_from(value: crate::Structure) -> Result<Self, Self::Error> {
        if let Structure::Statement { args } = value {
            match args.get(0).map(|s| format!("{}", s)).as_deref() {
                Some("error_log") => {
                    let path = args.get(1).ok_or(())?.to_string();
                    let path = PathBuf::from(path);
                    let level = args.get(2).map(|s| s.to_string());
                    return Ok(Self::ErrorLog { path, level })
                },
                Some("access_log") => {
                    let path = args.get(1).ok_or(())?.to_string();
                    let path = PathBuf::from(path);
                    return Ok(Self::AccessLog { path, compression_enabled: false })                    
                },
                Some("add_header") => {
                    let name = args.get(1).ok_or(())?.to_string();
                    let value = args.get(2..).ok_or(())?.iter().map(|s| format!(" {}", s.to_string())).collect::<String>();
                    return Ok(Self::AddHeader { name, value })
                },
                Some("auth_basic") => {
                    let realm = args.get(1) .ok_or(())?.to_string();
                    return Ok(Self::AuthBasic { realm })
                },
                Some("auth_basic_user_file") => {
                    let file = args.get(1) .ok_or(())?.to_string();
                    let file = PathBuf::from(file);
                    return Ok(Self::AuthBasicUserFile { file })                    
                },
                Some("http2") => {
                    let enabled = args.get(1).ok_or(())?.to_string().eq_ignore_ascii_case("on")
                                     || !args.get(1).ok_or(())?.to_string().eq_ignore_ascii_case("off");
                    return Ok(Self::Http2 { enabled })
                },
                Some("proxy_http_version") => {
                    let version = args.get(1).ok_or(())?.to_string();
                    return Ok(Self::ProxyHttpVersion { version })
                },
                Some("proxy_pass") => {
                    let addr = args.get(1).ok_or(())?.to_string();
                    let addr = Url::parse(&addr).map_err(|_| ())?;
                    return Ok(Self::ProxyPass { addr })
                },
                Some("proxy_hide_header") => {
                    let header_name = args.get(1).ok_or(())?.to_string();
                    return Ok(Self::ProxyHideHeader { header_name })
                },
                Some("proxy_set_header") => {
                    let header_name = args.get(1).ok_or(())?.to_string();
                    let header_value = args.get(2).ok_or(())?.to_string();
                    return Ok(Self::ProxySetHeader { header_name, header_value })
                },
                Some("server_name") => {
                    let name = args.get(2..).ok_or(())?.iter().map(|s| format!("{}", s.to_string())).collect::<String>();
                    return Ok(Self::ServerName { name })
                },
                Some("ssl_cerificate") => {
                    let path = PathBuf::from( args.get(1).ok_or(())?.to_string() );
                    return Ok(Self::SslCertificate { path })
                },
                Some("ssl_certificate_key") => {
                    let path = PathBuf::from( args.get(1).ok_or(())?.to_string() );
                    return Ok(Self::SslCertificateKey { path })                    
                },
                Some("listen") => {
                    let sock_addr = args.get(1).ok_or(())?.to_string();
                    let sock_addr = SocketAddr::from_str(&sock_addr).map_err(|_|())?;
                    let is_default = args.iter().any(|s| s.to_string().eq_ignore_ascii_case("default_server"));
                    let is_http2 = args.iter().any(|s| s.to_string().eq_ignore_ascii_case("http2"));
                    let is_http3 = args.iter().any(|s| s.to_string().eq_ignore_ascii_case("http3"));
                    return Ok(Self::Listen { sock_addr, is_default, is_http2, is_http3 })
                },
                
                // TODO
                
                _ => unreachable!()               
            };
        }
        
        Err(())
    }
}
                

