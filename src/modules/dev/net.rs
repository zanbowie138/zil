//! Networking math: IP addresses as numbers, CIDR blocks, subnetting.

use crate::ast::Radix;
use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, doc};
use crate::value::Value;
use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub const MODULE: Module = Module {
    name: "net",
    about: "IP addresses, CIDR blocks, subnets",
    #[rustfmt::skip]
    examples: &[
        ("net", &[
            ("hosts in a /22", r#"cidr("10.0.0.0/22").hosts"#),
            ("is it in the block?", r#"in_cidr("10.0.3.7", "10.0.0.0/22")"#),
            ("address as bits", r#"ip("192.168.1.5") to bin"#),
            ("next address", r#"ip(ip("10.0.0.255") + 1)"#),
            ("split into /24s", r#"subnets("10.0.0.0/22", 24)"#),
        ]),
    ],
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("ip", "ip(v: str|int)", "an IPv4 or IPv6 address as an integer, or an integer as an address", &[r#"ip("192.168.1.5")"#, "ip(3232235781)", r#"ip("::1")"#], &["cidr", "ip_kind"]),
    doc("cidr", "cidr(s: str)", "an IPv4 block's network, broadcast, netmask, host range and count", &[r#"cidr("192.168.1.77/26")"#], &["in_cidr", "subnets"]),
    doc("in_cidr", "in_cidr(ip: str, block: str)", "whether an IPv4 address is inside a CIDR block", &[r#"in_cidr("10.0.3.7", "10.0.0.0/22")"#], &["cidr"]),
    doc("subnets", "subnets(block: str, prefix: int)", "split an IPv4 block into smaller blocks", &[r#"subnets("10.0.0.0/24", 26)"#], &["cidr"]),
    doc("ip_kind", "ip_kind(ip: str)", "loopback, private, link-local, multicast, unspecified or public", &[r#"ip_kind("10.1.2.3")"#, r#"ip_kind("8.8.8.8")"#], &["ip"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("ip", [Str(s)]) => match addr(s)? {
            IpAddr::V4(a) => Value::int(u32::from(a) as i64),
            IpAddr::V6(a) => crate::value::exact(num_rational::BigRational::from_integer(BigInt::from(u128::from(a))), Radix::DEC, false),
        },
        ("ip", [Int(n, _)]) if (0..=u32::MAX as i64).contains(n) => Value::str(Ipv4Addr::from(*n as u32).to_string()),
        ("ip", [Int(n, _)]) if *n > 0 => Value::str(Ipv6Addr::from(*n as u128).to_string()),
        ("ip", [Big(n, _)]) => Value::str(Ipv6Addr::from(n.to_u128().ok_or("not a valid address")?).to_string()),
        ("cidr", [Str(s)]) => {
            let (net, prefix) = block(s)?;
            let size = 1u64 << (32 - prefix);
            let last = net as u64 + size - 1;
            // /31 and /32 have no network or broadcast address to set aside.
            let (first, hosts_last, hosts) = if prefix >= 31 { (net as u64, last, size) } else { (net as u64 + 1, last - 1, size - 2) };
            let show = |n: u64| Value::str(Ipv4Addr::from(n as u32).to_string());
            let mut m = IndexMap::new();
            m.insert("network".into(), show(net as u64));
            m.insert("broadcast".into(), show(last));
            m.insert("netmask".into(), show(mask(prefix) as u64));
            m.insert("prefix".into(), Value::int(prefix as i64));
            m.insert("first".into(), show(first));
            m.insert("last".into(), show(hosts_last));
            m.insert("hosts".into(), Value::int(hosts as i64));
            Value::map(m)
        }
        ("in_cidr", [Str(ip), Str(b)]) => {
            let (net, prefix) = block(b)?;
            let IpAddr::V4(a) = addr(ip)? else {
                return Err(Fail::Arg(0, "only IPv4 addresses are supported".into()));
            };
            Bool(u32::from(a) & mask(prefix) == net)
        }
        ("subnets", [Str(b), Int(p, _)]) => {
            let (net, prefix) = block(b)?;
            let p = u32::try_from(*p)
                .ok()
                .filter(|q| (prefix..=32).contains(q))
                .ok_or_else(|| Fail::Arg(1, format!("prefix must be from {prefix} to 32, got {p}")))?;
            if p - prefix > 12 {
                return Err(Fail::Arg(1, format!("{} subnets is too many to list\nnote: at most 4096 are listed", 1u64 << (p - prefix))));
            }
            let step = 1u64 << (32 - p);
            Value::list((0..1u64 << (p - prefix)).map(|i| Value::str(format!("{}/{p}", Ipv4Addr::from((net as u64 + i * step) as u32)))).collect())
        }
        ("ip_kind", [Str(s)]) => Value::str(match addr(s)? {
            IpAddr::V4(a) if a.is_loopback() => "loopback",
            IpAddr::V4(a) if a.is_private() => "private",
            IpAddr::V4(a) if a.is_link_local() => "link-local",
            IpAddr::V4(a) if a.is_multicast() => "multicast",
            IpAddr::V4(a) if a.is_unspecified() => "unspecified",
            IpAddr::V6(a) if a.is_loopback() => "loopback",
            IpAddr::V6(a) if a.is_multicast() => "multicast",
            IpAddr::V6(a) if a.is_unspecified() => "unspecified",
            IpAddr::V6(a) if a.segments()[0] & 0xfe00 == 0xfc00 => "private",
            IpAddr::V6(a) if a.segments()[0] & 0xffc0 == 0xfe80 => "link-local",
            _ => "public",
        }),
        _ => return Err(Fail::BadArgs),
    })
}

fn addr(s: &str) -> Result<IpAddr, String> {
    s.trim().parse().map_err(|_| format!("not an IP address: {s:?}"))
}

fn mask(prefix: u32) -> u32 {
    u32::MAX.checked_shl(32 - prefix).unwrap_or(0)
}

/// "10.0.0.0/22" as (network, prefix); host bits are cleared, so "10.0.3.7/22" works too.
fn block(s: &str) -> Result<(u32, u32), String> {
    let bad = || format!("not an IPv4 CIDR block: {s:?}");
    let (a, p) = s.trim().split_once('/').ok_or_else(bad)?;
    let a: Ipv4Addr = a.parse().map_err(|_| bad())?;
    let p: u32 = p.parse().ok().filter(|p| *p <= 32).ok_or_else(bad)?;
    Ok((u32::from(a) & mask(p), p))
}

#[cfg(test)]
mod tests {
    use crate::interp::tests::{show, try_eval};

    #[test]
    fn net() {
        assert_eq!(show(r#"ip("192.168.1.5")"#), "3232235781");
        assert_eq!(show("ip(3232235781)"), "192.168.1.5");
        assert_eq!(show(r#"ip(ip("2001:db8::1"))"#), "2001:db8::1");
        assert_eq!(show(r#"cidr("10.0.3.7/22").network"#), "10.0.0.0");
        assert_eq!(show(r#"cidr("10.0.0.0/22").hosts"#), "1022");
        assert_eq!(show(r#"cidr("10.0.0.0/22").broadcast"#), "10.0.3.255");
        assert_eq!(show(r#"cidr("10.0.0.0/22").netmask"#), "255.255.252.0");
        assert_eq!(show(r#"cidr("1.2.3.4/32").hosts"#), "1");
        assert_eq!(show(r#"cidr("0.0.0.0/0").hosts"#), "4294967294");
        assert_eq!(show(r#"in_cidr("10.0.3.7", "10.0.0.0/22")"#), "true");
        assert_eq!(show(r#"in_cidr("10.0.4.0", "10.0.0.0/22")"#), "false");
        assert_eq!(show(r#"subnets("10.0.0.0/23", 24)"#), r#"["10.0.0.0/24", "10.0.1.0/24"]"#);
        assert_eq!(show(r#"ip_kind("192.168.0.1")"#), "private");
        assert!(try_eval(r#"cidr("10.0.0.0/33")"#).is_err());
        assert!(try_eval(r#"ip("300.1.1.1")"#).is_err());
    }
}
