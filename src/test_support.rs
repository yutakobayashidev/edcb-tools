use crate::codec::{Writer, write_service_info};
use crate::types::ServiceInfo;

#[doc(hidden)]
pub fn encode_service_list_for_test() -> Vec<u8> {
    let service = ServiceInfo {
        onid: 1,
        tsid: 2,
        sid: 3,
        service_type: 1,
        partial_reception_flag: 0,
        service_provider_name: "Provider".to_string(),
        service_name: "Test Service".to_string(),
        network_name: "Network".to_string(),
        ts_name: "TS".to_string(),
        remote_control_key_id: 7,
    };

    let mut writer = Writer::new();
    writer.write_vector(&[service], write_service_info);
    writer.into_inner()
}
