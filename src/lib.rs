use std::net::Ipv4Addr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Endpoint { pub ip: Ipv4Addr, pub port: Option<u16> }

// Recognize only the same explicit connection markers used by the inspected tool.
// An explicit but malformed address invalidates a previous candidate.
pub fn latest_endpoint(text: &str) -> Option<Endpoint> {
    let mut last = None;
    for line in text.lines() {
        for marker in ["address=", "possible ip address:"] {
            if let Some((_, tail)) = line.split_once(marker) {
                let tail = tail.trim_start();
                let n = tail.bytes().take_while(|c| c.is_ascii_digit() || *c == b'.').count();
                last = tail[..n].parse::<Ipv4Addr>().ok().and_then(|ip| {
                    if ip.is_unspecified() || ip.is_loopback() || ip.is_multicast() || ip.is_broadcast() { return None; }
                    let rest = &tail[n..];
                    let port = if let Some(p) = rest.strip_prefix(':') {
                        let digits: String = p.chars().take_while(|c| c.is_ascii_digit()).collect();
                        match digits.parse::<u16>() { Ok(v) if v != 0 => Some(v), _ => return None }
                    } else { None };
                    Some(Endpoint { ip, port })
                });
            }
        }
    }
    last
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Connection {
    pub pid: u32, pub state: u32, pub local_addr: u32, pub local_port: u32,
    pub remote_addr: u32, pub remote_port: u32,
}
impl Connection {
    pub fn remote_ip(self) -> Ipv4Addr { Ipv4Addr::from(self.remote_addr.to_ne_bytes()) }
    pub fn remote_port_host(self) -> u16 { u16::from_be(self.remote_port as u16) }
}
// Never fall back to closing every Hearthstone/Battle.net connection.
pub fn select_connection(rows: &[Connection], pid: u32, ep: Endpoint) -> Result<Connection, String> {
    let mut candidates = rows.iter().copied().filter(|r| r.pid == pid && r.state == 5
        && r.remote_ip() == ep.ip && ep.port.map_or(true, |p| r.remote_port_host() == p));
    let first = candidates.next().ok_or("No matching established match connection")?;
    if candidates.next().is_some() { return Err("Ambiguous match connection; nothing disconnected".into()); }
    Ok(first)
}


// A previously identified target can survive quiet/rotated logs ONLY while
// the exact established TCP tuple and PID still exist. Never select another
// arbitrary connection when fresh discovery fails.
pub fn cached_connection_valid(rows: &[Connection], pid: u32, cached: Connection) -> bool {
    cached.pid == pid && cached.state == 5 && rows.iter().any(|r| *r == cached)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(pid:u32,ip:[u8;4],port:u16)->Connection {
        Connection {pid,state:5,local_addr:0,local_port:1,remote_addr:u32::from_ne_bytes(ip),remote_port:port.to_be() as u32}
    }
    #[test] fn explicit_address() { assert_eq!(latest_endpoint("D address= 203.0.113.8:1119"),Some(Endpoint{ip:Ipv4Addr::new(203,0,113,8),port:Some(1119)})); }
    #[test] fn alternate_marker() { assert_eq!(latest_endpoint("possible ip address: 203.0.113.9").unwrap().port,None); }
    #[test] fn last_wins() { assert_eq!(latest_endpoint("address=203.0.113.1:1\naddress=203.0.113.2:2").unwrap().port,Some(2)); }
    #[test] fn malformed_latest_invalidates() { assert!(latest_endpoint("address=203.0.113.1:1\naddress=999.2.2.2:2").is_none()); }
    #[test] fn unsafe_addresses_rejected() { for s in ["address=127.0.0.1:1","address=0.0.0.0:1","address=224.1.2.3:1","address=203.0.113.1:99999"] { assert!(latest_endpoint(s).is_none()); } }
    #[test] fn ignores_other_pid() { let a=row(1,[203,0,113,8],1119);let b=row(2,[203,0,113,8],1119);assert_eq!(select_connection(&[a,b],1,latest_endpoint("address=203.0.113.8:1119").unwrap()).unwrap(),a); }
    #[test] fn ambiguity_is_error() { let a=row(1,[203,0,113,8],1119);assert!(select_connection(&[a,a],1,latest_endpoint("address=203.0.113.8").unwrap()).is_err()); }
    #[test] fn non_established_rejected() { let mut a=row(1,[203,0,113,8],1119);a.state=2;assert!(select_connection(&[a],1,latest_endpoint("address=203.0.113.8").unwrap()).is_err()); }
    #[test] fn port_is_respected() { let a=row(1,[203,0,113,8],1119); assert!(select_connection(&[a],1,latest_endpoint("address=203.0.113.8:1120").unwrap()).is_err()); }
    #[test] fn cache_keeps_exact_live_tuple() {let a=row(1,[203,0,113,8],1119);assert!(cached_connection_valid(&[a],1,a));}
    #[test] fn cache_rejects_missing_tuple() {let a=row(1,[203,0,113,8],1119);assert!(!cached_connection_valid(&[],1,a));}
    #[test] fn cache_rejects_changed_local_port() {let a=row(1,[203,0,113,8],1119);let mut b=a;b.local_port=2;assert!(!cached_connection_valid(&[b],1,a));}
    #[test] fn cache_rejects_other_pid() {let a=row(1,[203,0,113,8],1119);assert!(!cached_connection_valid(&[a],2,a));}
    #[test] fn cache_rejects_non_established() {let mut a=row(1,[203,0,113,8],1119);a.state=8;assert!(!cached_connection_valid(&[a],1,a));}

}
