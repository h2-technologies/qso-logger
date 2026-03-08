pub const IPV6_PREFIX: [u16; 3] = [0x2602, 0xfa86, 0x0044];
/// The subnet identifier for the global unicast /64 prefix (`2602:fa86:44:44::/64`).
/// Used to generate station unicast addresses in the global addressing space.
pub const GLOBAL_UNICAST_SUBNET: u16 = 0x0044;
/// Full 64-bit network prefix embedded in RFC 3306 unicast-prefix-based multicast addresses.
/// Corresponds to `2602:fa86:44:44::/64` (plen = 0x40).
pub const MULTICAST_NETWORK_PREFIX: [u16; 4] = [0x2602, 0xfa86, 0x0044, 0x0044];
pub const MULTICAST_GROUP_ALL_STATIONS: u32 = 1;
/// Reverse DNS zone covering the /48 amateur-radio IPv6 prefix
/// (`2602:fa86:44::/48`).  PTR records for addresses within this prefix
/// are delegated under this zone.
pub const REVERSE_ZONE: &str = "0.6.8.a.f.2.0.6.2.ip6.arpa";

/// Encode a callsign into a 64-bit interface identifier using base-40 encoding.
///
/// Encoding table:
/// - 0 = null/terminator
/// - 1-26 = A-Z
/// - 27-36 = 0-9
/// - 37 = '/'
/// - 38 = '-'
/// - 39 = ' '
pub fn callsign_to_interface_id(callsign: &str) -> u64 {
    let mut id: u64 = 0;
    for (i, ch) in callsign.chars().enumerate() {
        if i >= 10 {
            break;
        }
        let val = char_to_base40(ch);
        id = id * 40 + val as u64;
    }
    id
}

fn char_to_base40(ch: char) -> u8 {
    match ch {
        'A'..='Z' => (ch as u8) - b'A' + 1,
        'a'..='z' => (ch as u8) - b'a' + 1,
        '0'..='9' => (ch as u8) - b'0' + 27,
        '/' => 37,
        '-' => 38,
        ' ' => 39,
        _ => 0,
    }
}

/// Decode a base-40 interface ID back to a callsign string.
pub fn interface_id_to_callsign(id: u64) -> String {
    if id == 0 {
        return String::new();
    }
    let mut remaining = id;
    let mut chars = Vec::new();
    while remaining > 0 {
        let rem = (remaining % 40) as u8;
        remaining /= 40;
        if rem == 0 {
            break;
        }
        chars.push(base40_to_char(rem));
    }
    chars.reverse();
    chars.into_iter().collect()
}

fn base40_to_char(val: u8) -> char {
    match val {
        1..=26 => (b'A' + val - 1) as char,
        27..=36 => (b'0' + val - 27) as char,
        37 => '/',
        38 => '-',
        39 => ' ',
        _ => '\0',
    }
}

/// Generate an IPv6 address for a callsign and subnet using the amateur radio prefix.
pub fn generate_ipv6_address(callsign: &str, subnet: u16) -> std::net::Ipv6Addr {
    let iface_id = callsign_to_interface_id(callsign);
    std::net::Ipv6Addr::new(
        IPV6_PREFIX[0],
        IPV6_PREFIX[1],
        IPV6_PREFIX[2],
        subnet,
        (iface_id >> 48) as u16,
        (iface_id >> 32) as u16,
        (iface_id >> 16) as u16,
        iface_id as u16,
    )
}

/// Generate a global-scope RFC 3306 unicast-prefix-based multicast address.
///
/// Layout (16-bit segments):
/// - seg[0]: `0xff3e` — multicast, flags P+T, global scope
/// - seg[1]: `0x0040` — reserved byte + plen=64 (for a /64 network prefix)
/// - seg[2..5]: `2602:fa86:44:44` — the 64-bit network prefix
/// - seg[6..7]: `group_id` split across high and low 16-bit halves
pub fn multicast_global(group_id: u32) -> std::net::Ipv6Addr {
    std::net::Ipv6Addr::new(
        0xff3e,
        0x0040,
        MULTICAST_NETWORK_PREFIX[0],
        MULTICAST_NETWORK_PREFIX[1],
        MULTICAST_NETWORK_PREFIX[2],
        MULTICAST_NETWORK_PREFIX[3],
        (group_id >> 16) as u16,
        group_id as u16,
    )
}

/// Generate a site-local-scope RFC 3306 unicast-prefix-based multicast address.
///
/// See [`multicast_global`] for the segment layout; scope nibble is `5` (site-local).
pub fn multicast_site_local(group_id: u32) -> std::net::Ipv6Addr {
    std::net::Ipv6Addr::new(
        0xff35,
        0x0040,
        MULTICAST_NETWORK_PREFIX[0],
        MULTICAST_NETWORK_PREFIX[1],
        MULTICAST_NETWORK_PREFIX[2],
        MULTICAST_NETWORK_PREFIX[3],
        (group_id >> 16) as u16,
        group_id as u16,
    )
}

/// Generate a link-local-scope RFC 3306 unicast-prefix-based multicast address.
///
/// See [`multicast_global`] for the segment layout; scope nibble is `2` (link-local).
pub fn multicast_link_local(group_id: u32) -> std::net::Ipv6Addr {
    std::net::Ipv6Addr::new(
        0xff32,
        0x0040,
        MULTICAST_NETWORK_PREFIX[0],
        MULTICAST_NETWORK_PREFIX[1],
        MULTICAST_NETWORK_PREFIX[2],
        MULTICAST_NETWORK_PREFIX[3],
        (group_id >> 16) as u16,
        group_id as u16,
    )
}

fn addr_nibbles(addr: &std::net::Ipv6Addr) -> [u8; 32] {
    let octets = addr.octets();
    let mut nibbles = [0u8; 32];
    for (i, &byte) in octets.iter().enumerate() {
        nibbles[i * 2] = (byte >> 4) & 0xf;
        nibbles[i * 2 + 1] = byte & 0xf;
    }
    nibbles
}

/// Generate the full reverse DNS name for an IPv6 address (ending in `.ip6.arpa`).
pub fn reverse_dns_name(addr: &std::net::Ipv6Addr) -> String {
    let nibbles = addr_nibbles(addr);
    let reversed: Vec<String> = nibbles.iter().rev().map(|n| format!("{:x}", n)).collect();
    format!("{}.ip6.arpa", reversed.join("."))
}

/// Generate the record name relative to zone `0.6.8.a.f.2.0.6.2.ip6.arpa`.
///
/// Returns the first 23 nibbles of the reversed address joined by dots.
pub fn reverse_dns_record_name(addr: &std::net::Ipv6Addr) -> String {
    let nibbles = addr_nibbles(addr);
    let reversed: Vec<String> = nibbles
        .iter()
        .rev()
        .take(23)
        .map(|n| format!("{:x}", n))
        .collect();
    reversed.join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callsign_to_interface_id_w1aw() {
        let id = callsign_to_interface_id("W1AW");
        assert_ne!(id, 0);
        // W=23, 1=28, A=1, W=23
        // id = ((23*40 + 28)*40 + 1)*40 + 23
        let expected: u64 = ((23u64 * 40 + 28) * 40 + 1) * 40 + 23;
        assert_eq!(id, expected);
    }

    #[test]
    fn test_callsign_to_interface_id_k6xx() {
        let id = callsign_to_interface_id("K6XX");
        assert_ne!(id, 0);
        // K=11, 6=33, X=24, X=24
        let expected: u64 = ((11u64 * 40 + 33) * 40 + 24) * 40 + 24;
        assert_eq!(id, expected);
    }

    #[test]
    fn test_callsign_to_interface_id_n0call() {
        let id = callsign_to_interface_id("N0CALL");
        assert_ne!(id, 0);
        // N=14, 0=27, C=3, A=1, L=12, L=12
        let expected: u64 = (((((14u64 * 40 + 27) * 40 + 3) * 40 + 1) * 40 + 12) * 40) + 12;
        assert_eq!(id, expected);
    }

    #[test]
    fn test_round_trip_w1aw() {
        let callsign = "W1AW";
        let id = callsign_to_interface_id(callsign);
        let decoded = interface_id_to_callsign(id);
        assert_eq!(decoded, callsign);
    }

    #[test]
    fn test_round_trip_k6xx() {
        let callsign = "K6XX";
        let id = callsign_to_interface_id(callsign);
        let decoded = interface_id_to_callsign(id);
        assert_eq!(decoded, callsign);
    }

    #[test]
    fn test_round_trip_n0call() {
        let callsign = "N0CALL";
        let id = callsign_to_interface_id(callsign);
        let decoded = interface_id_to_callsign(id);
        assert_eq!(decoded, callsign);
    }

    #[test]
    fn test_interface_id_zero_returns_empty() {
        assert_eq!(interface_id_to_callsign(0), "");
    }

    #[test]
    fn test_generate_ipv6_address_prefix() {
        let addr = generate_ipv6_address("W1AW", 0);
        let segs = addr.segments();
        assert_eq!(segs[0], 0x2602);
        assert_eq!(segs[1], 0xfa86);
        assert_eq!(segs[2], 0x0044);
        assert_eq!(segs[3], 0x0000);
    }

    #[test]
    fn test_generate_ipv6_address_global_unicast_subnet() {
        let addr = generate_ipv6_address("W1AW", GLOBAL_UNICAST_SUBNET);
        let segs = addr.segments();
        assert_eq!(segs[0], 0x2602);
        assert_eq!(segs[1], 0xfa86);
        assert_eq!(segs[2], 0x0044);
        assert_eq!(segs[3], 0x0044);
    }

    #[test]
    fn test_generate_ipv6_address_with_subnet() {
        let addr = generate_ipv6_address("W1AW", 5);
        let segs = addr.segments();
        assert_eq!(segs[0], 0x2602);
        assert_eq!(segs[1], 0xfa86);
        assert_eq!(segs[2], 0x0044);
        assert_eq!(segs[3], 5);
    }

    #[test]
    fn test_multicast_global() {
        let addr = multicast_global(MULTICAST_GROUP_ALL_STATIONS);
        let segs = addr.segments();
        assert_eq!(segs[0], 0xff3e);
        assert_eq!(segs[1], 0x0040);
        assert_eq!(segs[2], 0x2602);
        assert_eq!(segs[3], 0xfa86);
        assert_eq!(segs[4], 0x0044);
        assert_eq!(segs[5], 0x0044);
        assert_eq!(segs[6], (MULTICAST_GROUP_ALL_STATIONS >> 16) as u16);
        assert_eq!(segs[7], MULTICAST_GROUP_ALL_STATIONS as u16);
    }

    #[test]
    fn test_multicast_global_large_group_id() {
        // Verify that the high 16 bits of a >16-bit group_id are preserved.
        let group_id: u32 = 0x0001_0002;
        let addr = multicast_global(group_id);
        let segs = addr.segments();
        assert_eq!(segs[0], 0xff3e);
        assert_eq!(segs[1], 0x0040);
        assert_eq!(segs[5], 0x0044);
        assert_eq!(segs[6], 0x0001);
        assert_eq!(segs[7], 0x0002);
    }

    #[test]
    fn test_multicast_site_local() {
        let addr = multicast_site_local(MULTICAST_GROUP_ALL_STATIONS);
        let segs = addr.segments();
        assert_eq!(segs[0], 0xff35);
        assert_eq!(segs[1], 0x0040);
        assert_eq!(segs[5], 0x0044);
        assert_eq!(segs[6], (MULTICAST_GROUP_ALL_STATIONS >> 16) as u16);
        assert_eq!(segs[7], MULTICAST_GROUP_ALL_STATIONS as u16);
    }

    #[test]
    fn test_multicast_link_local() {
        let addr = multicast_link_local(MULTICAST_GROUP_ALL_STATIONS);
        let segs = addr.segments();
        assert_eq!(segs[0], 0xff32);
        assert_eq!(segs[1], 0x0040);
        assert_eq!(segs[5], 0x0044);
        assert_eq!(segs[6], (MULTICAST_GROUP_ALL_STATIONS >> 16) as u16);
        assert_eq!(segs[7], MULTICAST_GROUP_ALL_STATIONS as u16);
    }

    #[test]
    fn test_reverse_dns_name() {
        let addr: std::net::Ipv6Addr = "2602:fa86:44:0:1234:5678:9abc:def0".parse().unwrap();
        let name = reverse_dns_name(&addr);
        assert_eq!(
            name,
            "0.f.e.d.c.b.a.9.8.7.6.5.4.3.2.1.0.0.0.0.4.4.0.0.6.8.a.f.2.0.6.2.ip6.arpa"
        );
    }

    #[test]
    fn test_reverse_dns_record_name() {
        let addr: std::net::Ipv6Addr = "2602:fa86:44:0:1234:5678:9abc:def0".parse().unwrap();
        let record = reverse_dns_record_name(&addr);
        assert_eq!(record, "0.f.e.d.c.b.a.9.8.7.6.5.4.3.2.1.0.0.0.0.4.4.0");
    }
}
