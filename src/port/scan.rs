use local_ip_address::local_ip;
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

#[cfg(test)]
mod test {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn test_scan_loop_back_ports() {
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let test_port = 1500;

        let _listener =
            TcpListener::bind(format!("{}:{}", ip, test_port)).expect("Failed to bind test port");

        let open_ports = scan_renge("127.0.0.1".into(), 1u16, 12000u16);
        println!("Open ports found: {:?}", open_ports);

        assert!(
            open_ports.contains(&test_port),
            "Scanner missed the open port on local IP!"
        );
    }
    #[test]
    fn scan_on_local_ports() {
        let ip = local_ip_address::local_ip().unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        let test_port = 1500;

        let _listener =
            TcpListener::bind(format!("{}:{}", ip, test_port)).expect("Failed to bind test port");
        let open_port = sapl();
        println!("{:?}", open_port);
        assert!(
            open_port.contains(&test_port),
            "Scanner missed the open port on local IP!"
        );
    }
}
