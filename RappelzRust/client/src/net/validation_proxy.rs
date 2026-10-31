//! Explicit loopback-only route for native fault-injection tests; never rewrites advertised realms.
use anyhow::{ensure, Context, Result};
use std::net::{IpAddr, SocketAddr};

pub(super) fn endpoint(
    host: &str,
    advertised: (&str, u16),
    account: &str,
    character: Option<&str>,
    scenario: bool,
    proxy: Option<&str>,
) -> Result<(String, u16)> {
    let Some(proxy) = proxy else {
        return Ok((advertised.0.to_owned(), advertised.1));
    };
    let loopback =
        |s: &str| s == "localhost" || s.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback());
    ensure!(scenario && loopback(host) && loopback(advertised.0)
        && account.starts_with("LUA") && character.is_some_and(|s|s.starts_with("Lua")),
        "validation proxy requires the explicit local terrain-network scenario and dedicated account");
    let addr: SocketAddr = proxy.parse().context("numeric validation proxy address")?;
    ensure!(
        addr.ip().is_loopback() && addr.port() != 0,
        "validation proxy must be loopback with a nonzero port"
    );
    Ok((addr.ip().to_string(), addr.port()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_sessions_keep_the_advertised_endpoint() {
        assert_eq!(
            endpoint("auth", ("game", 4514), "player", None, false, None).unwrap(),
            ("game".into(), 4514)
        );
    }
    #[test]
    fn test_routes_require_dedicated_local_sessions() {
        for (host, server, account, character, scenario) in [
            ("remote", "127.0.0.1", "LUA123", Some("LuaTest"), true),
            ("127.0.0.1", "example.org", "LUA123", Some("LuaTest"), true),
            ("127.0.0.1", "127.0.0.1", "PLAYER", Some("LuaTest"), true),
            ("127.0.0.1", "127.0.0.1", "LUA123", Some("Player"), true),
            ("127.0.0.1", "127.0.0.1", "LUA123", Some("LuaTest"), false),
        ] {
            assert!(endpoint(
                host,
                (server, 4514),
                account,
                character,
                scenario,
                Some("127.0.0.1:50400")
            )
            .is_err());
        }
    }
    #[test]
    fn proxy_addresses_are_numeric_loopback_only() {
        for proxy in [
            "0.0.0.0:20",
            "192.168.1.1:20",
            "127.0.0.1:0",
            "remote:20",
            "invalid",
        ] {
            assert!(endpoint(
                "localhost",
                ("127.0.0.1", 4514),
                "LUA123",
                Some("LuaTest"),
                true,
                Some(proxy)
            )
            .is_err());
        }
        assert_eq!(
            endpoint(
                "localhost",
                ("127.0.0.1", 4514),
                "LUA123",
                Some("LuaTest"),
                true,
                Some("127.0.0.1:50400")
            )
            .unwrap(),
            ("127.0.0.1".into(), 50400)
        );
    }
}
