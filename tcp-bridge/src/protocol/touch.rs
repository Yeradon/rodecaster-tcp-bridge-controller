use super::RodeCommand;

pub struct ScreenTouched;
impl RodeCommand for ScreenTouched {
    fn build_payload(&self, session_id: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(session_id);
        payload.push(0x07); 
        payload.extend_from_slice(b"screenTouched\0"); 
        payload.push(0x01);
        payload.push(0x01);
        payload.push(0x02);
        payload
    }
}
