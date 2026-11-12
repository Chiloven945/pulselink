//! URL and QR helpers for local pairing.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use url::Url;

use crate::config::RelayConfig;

/// Returns the controller URL, using the configured public URL when present.
pub(crate) fn controller_url(config: &RelayConfig, addr: SocketAddr) -> String {
    if let Some(public) = &config.public_base_url {
        return format!(
            "{}{}",
            public.trim_end_matches('/'),
            normalized_path(&config.path)
        );
    }

    let ip = if addr.ip().is_unspecified() {
        detect_lan_ip().unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
    } else {
        addr.ip()
    };
    let host = match ip {
        IpAddr::V4(ip) => ip.to_string(),
        IpAddr::V6(ip) => format!("[{ip}]"),
    };
    format!(
        "ws://{host}:{}{}",
        addr.port(),
        normalized_path(&config.path)
    )
}

/// Appends a `tid` query parameter to the controller URL.
pub(crate) fn pairing_url(controller_url: &str, target_id: &str) -> String {
    if let Ok(mut url) = Url::parse(controller_url) {
        url.query_pairs_mut().append_pair("tid", target_id);
        return url.to_string();
    }

    let separator = if controller_url.contains('?') {
        '&'
    } else {
        '?'
    };
    format!("{controller_url}{separator}tid={}", encode(target_id))
}

/// Builds the official DG-LAB QR redirect URL.
pub(crate) fn qr_url(pairing_url: &str) -> String {
    let mut url = Url::parse("https://dungeon-lab.cn/s/").expect("static QR URL is valid");
    url.query_pairs_mut()
        .append_pair("v", "1")
        .append_pair("action", "socket")
        .append_pair("url", pairing_url);
    url.to_string()
}

fn normalized_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("/{path}")
    }
}

fn encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn detect_lan_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let address = socket.local_addr().ok()?;
    if address.ip().is_loopback() || address.ip().is_unspecified() {
        None
    } else {
        Some(address.ip())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_pairing_parameter() {
        let url = pairing_url("ws://127.0.0.1:8787/v4", "abc def");
        assert!(url.starts_with("ws://127.0.0.1:8787/v4?"));
        assert!(url.contains("tid=abc+def"));
    }

    #[test]
    fn public_url_overrides_detection() {
        let config = RelayConfig {
            public_base_url: Some("wss://relay.example/".to_owned()),
            ..RelayConfig::default()
        };
        assert_eq!(
            controller_url(&config, "127.0.0.1:1234".parse().expect("addr")),
            "wss://relay.example/v4"
        );
    }

    #[test]
    fn qr_url_encodes_pairing_url() {
        let qr = qr_url("ws://127.0.0.1:8787/v4?tid=abc");
        assert!(qr.starts_with("https://dungeon-lab.cn/s/?"));
        assert!(qr.contains("action=socket"));
        assert!(qr.contains("url=ws%3A%2F%2F127.0.0.1%3A8787%2Fv4%3Ftid%3Dabc"));
    }
}
