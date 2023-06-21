use std::net::UdpSocket;
use stunclient::StunClient;
use std::net::{ToSocketAddrs};

pub struct Stun{}
impl Stun{
    pub fn get_address_ipv6() -> String{
        let stun_addr = "stun.l.google.com:19302".to_socket_addrs().unwrap().filter(|x|x.is_ipv6()).next().unwrap();
        let udp = UdpSocket::bind("[::]:0").unwrap();
        let mut c = StunClient::new(stun_addr);
        c.set_software(Some("Savi"));
        c.query_external_address(&udp).unwrap().to_string()
    }
    pub fn get_address_ipv4() -> String{
        let stun_addr = "stun.l.google.com:19302".to_socket_addrs().unwrap().filter(|x|x.is_ipv4()).next().unwrap();
        let udp = UdpSocket::bind("[::]:0").unwrap();
        let mut c = StunClient::new(stun_addr);
        c.set_software(Some("Savi"));
        c.query_external_address(&udp).unwrap().to_string()
    }
}