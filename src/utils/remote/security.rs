use crate::error::{DeclarchError, Result};
use crate::project_identity;
use std::net::{IpAddr, ToSocketAddrs};

const SECURE_SCHEME: &str = "https";
const INSECURE_SCHEME: &str = "http";

pub(super) fn validate_url(url_str: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url_str)
        .map_err(|_| DeclarchError::RemoteFetchError(format!("Invalid URL: {}", url_str)))?;

    let scheme = parsed.scheme();
    if !is_allowed_scheme(scheme) {
        let insecure_key = project_identity::env_key("ALLOW_INSECURE_HTTP");
        return Err(DeclarchError::RemoteFetchError(format!(
            "URL scheme '{}' is blocked. Allowed by default: https. To allow http explicitly set {}=1.",
            scheme, insecure_key
        )));
    }

    let host = parsed.host_str().ok_or_else(|| {
        DeclarchError::RemoteFetchError(format!("URL must include a valid host: {}", url_str))
    })?;
    if is_private_address(host) {
        return Err(DeclarchError::RemoteFetchError(format!(
            "Access to private addresses is not allowed: {}",
            host
        )));
    }

    let port = parsed.port_or_known_default().unwrap_or(443);
    let resolved = resolve_host_addresses(host, port).map_err(|e| {
        DeclarchError::RemoteFetchError(format!("Failed to resolve host '{}': {}", host, e))
    })?;
    if resolved.is_empty() {
        return Err(DeclarchError::RemoteFetchError(format!(
            "Failed to resolve host '{}': no addresses returned",
            host
        )));
    }
    if let Some(private_ip) = first_private_ip(&resolved) {
        return Err(DeclarchError::RemoteFetchError(format!(
            "Access to private addresses is not allowed: {} -> {}",
            host, private_ip
        )));
    }

    Ok(())
}

fn is_allowed_scheme(scheme: &str) -> bool {
    if scheme == SECURE_SCHEME {
        return true;
    }

    if scheme == INSECURE_SCHEME {
        return project_identity::env_get("ALLOW_INSECURE_HTTP").unwrap_or_default() == "1";
    }

    false
}

fn resolve_host_addresses(host: &str, port: u16) -> std::io::Result<Vec<IpAddr>> {
    (host, port)
        .to_socket_addrs()
        .map(|iter| iter.map(|sa| sa.ip()).collect())
}

pub(super) fn first_private_ip(addrs: &[IpAddr]) -> Option<IpAddr> {
    addrs.iter().copied().find(|ip| is_private_ip(*ip))
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            ipv4.is_private() || ipv4.is_loopback() || ipv4.is_link_local() || ipv4.is_unspecified()
        }
        IpAddr::V6(ipv6) => {
            ipv6.is_loopback()
                || ipv6.is_unique_local()
                || ipv6.is_unicast_link_local()
                || ipv6.is_unspecified()
        }
    }
}

pub(super) fn is_private_address(host: &str) -> bool {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_private_ip(ip);
    }

    host.eq_ignore_ascii_case("localhost")
}
