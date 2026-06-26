use local_ip_address::local_ip;
use netstat2::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::net::{IpAddr, Ipv4Addr, TcpStream};
use std::time::Duration;

const START_PORTS: u16 = 1u16;
const END_PORTS: u16 = 65_535u16;
const TIMEOUT: Duration = Duration::from_millis(100);

//scan all ports local
pub fn sapl() -> Vec<u16> {
    let ip: IpAddr = local_ip().unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    sapoa(ip.to_string())
}

//scan all port on a address
pub fn sapoa(addr: String) -> Vec<u16> {
    scan_renge(addr, START_PORTS, END_PORTS)
}

//scan on range ports
pub fn scan_renge(addr: String, sp: u16, ep: u16) -> Vec<u16> {
    let open_port: Vec<u16> = (sp..=ep)
        .into_par_iter()
        .filter_map(|port| {
            let address = format!("{}:{}", addr, port);
            match TcpStream::connect_timeout(&address.parse().unwrap(), TIMEOUT) {
                Ok(_) => Some(port),
                Err(_) => None,
            }
        })
        .collect();
    open_port
}

//Scan TCP Ports with netstat2
pub fn scan_netstate_tcp() -> Vec<String> {
    let mut out_list = vec![];

    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP;
    let sockets_info = get_sockets_info(af_flags, proto_flags).unwrap();

    for si in sockets_info {
        if let ProtocolSocketInfo::Tcp(tcp_si) = si.protocol_socket_info {
            let data = format!(
                "TCP {}:{} -> {}:{} {:?} - {}",
                tcp_si.local_addr,
                tcp_si.local_port,
                tcp_si.remote_addr,
                tcp_si.remote_port,
                si.associated_pids,
                tcp_si.state
            );
            out_list.push(data);
        }
    }
    out_list
}

//Scan UDP Porst with netstat2
pub fn scan_netstate_udp() -> Vec<String> {
    let mut out_list = vec![];
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::UDP;
    let sockets_info = get_sockets_info(af_flags, proto_flags).unwrap();
    for si in sockets_info {
        if let ProtocolSocketInfo::Udp(udp_si) = si.protocol_socket_info {
            let data = format!(
                "UDP {}:{} -> *:* {:?}",
                udp_si.local_addr, udp_si.local_port, si.associated_pids
            );
            out_list.push(data);
        }
    }
    out_list
}

#[cfg(test)]
mod test {
    use crate::port::scan::{scan_netstate_tcp, scan_netstate_udp};

    #[test]
    fn test_tcp() {
        let status = scan_netstate_tcp();
        status.iter().for_each(|p| {
            println!("{:?}", p);
        });
    }
    #[test]
    fn test_udp() {
        let status = scan_netstate_udp();
        status.iter().for_each(|p| {
            println!("{:?}", p);
        });
    }
}
