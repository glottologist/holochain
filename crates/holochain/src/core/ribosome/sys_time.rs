use super::HostContext;
use super::WasmRibosome;
use std::sync::Arc;
use sx_zome_types::SysTimeInput;
use sx_zome_types::SysTimeOutput;
use std::net::{ToSocketAddrs, UdpSocket};
use roughenough::merkle::root_from_paths;
use roughenough::sign::Verifier;
use roughenough::{
    RtMessage, Tag, CERTIFICATE_CONTEXT, SIGNED_RESPONSE_CONTEXT,
};
use std::collections::HashMap;
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::offset::Utc;
use chrono::offset::TimeZone;

fn create_nonce() -> [u8; 64] {
    let nonce = [0u8; 64];
    nonce
}

fn make_request(nonce: &[u8]) -> Vec<u8> {
    let mut msg = RtMessage::new(1);
    msg.add_field(Tag::NONC, nonce).unwrap();
    msg.pad_to_kilobyte();

    msg.encode().unwrap()
}

fn receive_response(sock: &mut UdpSocket) -> RtMessage {
    let mut buf = [0; 744];
    let resp_len = sock.recv_from(&mut buf).unwrap().0;

    RtMessage::from_bytes(&buf[0..resp_len]).unwrap()
}

struct ResponseHandler {
    pub_key: Option<Vec<u8>>,
    msg: HashMap<Tag, Vec<u8>>,
    srep: HashMap<Tag, Vec<u8>>,
    cert: HashMap<Tag, Vec<u8>>,
    dele: HashMap<Tag, Vec<u8>>,
    nonce: [u8; 64],
}

pub struct ParsedResponse {
    verified: bool,
    midpoint: u64,
    radius: u32,
}

impl ResponseHandler {
    pub fn new(pub_key: Option<Vec<u8>>, response: RtMessage, nonce: [u8; 64]) -> ResponseHandler {
        let msg = response.into_hash_map();
        let srep = RtMessage::from_bytes(&msg[&Tag::SREP])
            .unwrap()
            .into_hash_map();
        let cert = RtMessage::from_bytes(&msg[&Tag::CERT])
            .unwrap()
            .into_hash_map();
        let dele = RtMessage::from_bytes(&cert[&Tag::DELE])
            .unwrap()
            .into_hash_map();

        ResponseHandler {
            pub_key,
            msg,
            srep,
            cert,
            dele,
            nonce,
        }
    }

    pub fn extract_time(&self) -> ParsedResponse {
        let midpoint = self.srep[&Tag::MIDP]
            .as_slice()
            .read_u64::<LittleEndian>()
            .unwrap();
        let radius = self.srep[&Tag::RADI]
            .as_slice()
            .read_u32::<LittleEndian>()
            .unwrap();

        let verified = if self.pub_key.is_some() {
            self.validate_dele();
            self.validate_srep();
            self.validate_merkle();
            self.validate_midpoint(midpoint);
            true
        } else {
            false
        };

        ParsedResponse {
            verified,
            midpoint,
            radius,
        }
    }

    fn validate_dele(&self) {
        let mut full_cert = Vec::from(CERTIFICATE_CONTEXT.as_bytes());
        full_cert.extend(&self.cert[&Tag::DELE]);

        assert!(
            self.validate_sig(
                self.pub_key.as_ref().unwrap(),
                &self.cert[&Tag::SIG],
                &full_cert
            ),
            "Invalid signature on DELE tag, response may not be authentic"
        );
    }

    fn validate_srep(&self) {
        let mut full_srep = Vec::from(SIGNED_RESPONSE_CONTEXT.as_bytes());
        full_srep.extend(&self.msg[&Tag::SREP]);

        assert!(
            self.validate_sig(&self.dele[&Tag::PUBK], &self.msg[&Tag::SIG], &full_srep),
            "Invalid signature on SREP tag, response may not be authentic"
        );
    }

    fn validate_merkle(&self) {
        let srep = RtMessage::from_bytes(&self.msg[&Tag::SREP])
            .unwrap()
            .into_hash_map();
        let index = self.msg[&Tag::INDX]
            .as_slice()
            .read_u32::<LittleEndian>()
            .unwrap();
        let paths = &self.msg[&Tag::PATH];

        let hash = root_from_paths(index as usize, &self.nonce, paths);

        assert_eq!(
            hash, srep[&Tag::ROOT],
            "Nonce is not present in the response's merkle tree"
        );
    }

    fn validate_midpoint(&self, midpoint: u64) {
        let mint = self.dele[&Tag::MINT]
            .as_slice()
            .read_u64::<LittleEndian>()
            .unwrap();
        let maxt = self.dele[&Tag::MAXT]
            .as_slice()
            .read_u64::<LittleEndian>()
            .unwrap();

        assert!(
            midpoint >= mint,
            "Response midpoint {} lies *before* delegation span ({}, {})",
            midpoint, mint, maxt
        );
        assert!(
            midpoint <= maxt,
            "Response midpoint {} lies *after* delegation span ({}, {})",
            midpoint, mint, maxt
        );
    }

    fn validate_sig(&self, public_key: &[u8], sig: &[u8], data: &[u8]) -> bool {
        let mut verifier = Verifier::new(public_key);
        verifier.update(data);
        verifier.verify(sig)
    }
}

pub fn sys_time(
    _ribosome: Arc<WasmRibosome>,
    _host_context: Arc<HostContext>,
    _input: SysTimeInput,
) -> SysTimeOutput {

    let addr_str = "roughtime.cloudflare.com:2002";
    let num_requests = 3;
    let pub_key: &[u8; 32] = &[0x80, 0x3e, 0xb7, 0x85, 0x28, 0xf7, 0x49, 0xc4, 0xbe, 0xc2, 0xe3, 0x9e, 0x1a, 0xbb, 0x9b, 0x5e, 0x5a, 0xb7, 0xe4, 0xdd, 0x5c, 0xe4, 0xb6, 0xf2, 0xfd, 0x2f, 0x93, 0xec, 0xc3, 0x53, 0x8f, 0x1a];

    let addr = addr_str.to_socket_addrs().unwrap().next().unwrap();

    let mut requests = Vec::with_capacity(num_requests);
    for _ in 0..num_requests {
        let nonce = create_nonce();
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Couldn't open UDP socket");
        let request = make_request(&nonce);
        requests.push((nonce, request, socket));
    }

    for &mut (_, ref request, ref mut socket) in &mut requests {
        socket.send_to(request, addr).unwrap();
    }

    for (nonce, _, mut socket) in requests {
        let resp = receive_response(&mut socket);

        let ParsedResponse {
            verified,
            midpoint,
            radius,
        } = ResponseHandler::new(Some(pub_key.to_vec()), resp.clone(), nonce).extract_time();

        dbg!("x: {} {} {}", verified, midpoint, radius);

        let map = resp.into_hash_map();
        let _index = map[&Tag::INDX]
        .as_slice()
        .read_u32::<LittleEndian>()
        .unwrap();

        let seconds = midpoint / 10_u64.pow(6);
        let nsecs = (midpoint - (seconds * 10_u64.pow(6))) * 10_u64.pow(3);

        let ts = Utc.timestamp(seconds as i64, nsecs as u32);
        dbg!("y: {:?}", ts);
    }

    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    SysTimeOutput::new(since_the_epoch)
}

#[cfg(test)]
pub mod wasm_test {
    use sx_zome_types::zome_io::SysTimeOutput;
    use sx_zome_types::SysTimeInput;

    #[test]
    fn invoke_import_sys_time_test() {
        let _: SysTimeOutput =
            crate::call_test_ribosome!("imports", "sys_time", SysTimeInput::new(()));
    }
}
