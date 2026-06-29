use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time;

use crate::codec::{
    Reader, Writer, read_auto_add_data, read_event_info, read_file_data, read_manual_auto_add_data,
    read_notify_srv_info, read_nw_play_time_shift_info, read_rec_file_info, read_reserve_data,
    read_service_event_info, read_service_info, read_tuner_process_status_info,
    read_tuner_reserve_info, write_i64_vector, write_reserve_data, write_search_key_info,
    write_string_vector,
};
use crate::error::{EdcbError, Result};
use crate::types::*;

const CMD_SUCCESS: i32 = 1;
const CMD_VER: u16 = 5;
const CMD_EPG_SRV_DEL_RESERVE: i32 = 1014;
const CMD_EPG_SRV_ENUM_TUNER_RESERVE: i32 = 1016;
const CMD_EPG_SRV_DEL_RECINFO: i32 = 1018;
const CMD_EPG_SRV_CHG_PATH_RECINFO: i32 = 1019;
const CMD_EPG_SRV_ENUM_SERVICE: i32 = 1021;
const CMD_EPG_SRV_SEARCH_PG: i32 = 1025;
const CMD_EPG_SRV_ENUM_PG_INFO_EX: i32 = 1029;
const CMD_EPG_SRV_ENUM_PG_ARC: i32 = 1030;
const CMD_EPG_SRV_DEL_AUTO_ADD: i32 = 1033;
const CMD_EPG_SRV_DEL_MANU_ADD: i32 = 1043;
const CMD_EPG_SRV_EPG_CAP_NOW: i32 = 1053;
const CMD_EPG_SRV_FILE_COPY: i32 = 1060;
const CMD_EPG_SRV_ENUM_PLUGIN: i32 = 1061;
const CMD_EPG_SRV_ENUM_TUNER_PROCESS: i32 = 1066;
const CMD_EPG_SRV_NWPLAY_CLOSE: i32 = 1081;
const CMD_EPG_SRV_NWPLAY_TF_OPEN: i32 = 1087;
const CMD_EPG_SRV_GET_NETWORK_PATH: i32 = 1299;
const CMD_EPG_SRV_ENUM_RESERVE2: i32 = 2011;
const CMD_EPG_SRV_GET_RESERVE2: i32 = 2012;
const CMD_EPG_SRV_ADD_RESERVE2: i32 = 2013;
const CMD_EPG_SRV_CHG_RESERVE2: i32 = 2015;
const CMD_EPG_SRV_CHG_PROTECT_RECINFO2: i32 = 2019;
const CMD_EPG_SRV_ENUM_RECINFO_BASIC2: i32 = 2020;
const CMD_EPG_SRV_GET_RECINFO2: i32 = 2024;
const CMD_EPG_SRV_FILE_COPY2: i32 = 2060;
const CMD_EPG_SRV_ENUM_AUTO_ADD2: i32 = 2131;
const CMD_EPG_SRV_ADD_AUTO_ADD2: i32 = 2132;
const CMD_EPG_SRV_CHG_AUTO_ADD2: i32 = 2134;
const CMD_EPG_SRV_ENUM_MANU_ADD2: i32 = 2141;
const CMD_EPG_SRV_ADD_MANU_ADD2: i32 = 2142;
const CMD_EPG_SRV_CHG_MANU_ADD2: i32 = 2144;
const CMD_EPG_SRV_GET_STATUS_NOTIFY2: i32 = 2200;
const EPG_SERVICE_ALL_MASK: i64 = 0x0000_ffff_ffff_ffff;
const EPG_LOOKUP_TIME_BEGIN: i64 = 1;
const EPG_LOOKUP_TIME_END: i64 = i64::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginKind {
    RecName = 1,
    Write = 2,
}

const DEFAULT_RESERVE_ID: i32 = 0x7fff_ffff;

#[derive(Debug, Clone)]
pub struct EdcbClient {
    host: String,
    port: u16,
    timeout: Duration,
}

impl EdcbClient {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            timeout: Duration::from_secs(15),
        }
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    pub async fn enum_service(&self) -> Result<Vec<ServiceInfo>> {
        let body = self.send_cmd(CMD_EPG_SRV_ENUM_SERVICE, |_| {}).await?;
        let mut reader = Reader::new(&body);
        reader.read_vector(read_service_info)
    }

    pub async fn enum_pg_info_ex(
        &self,
        service_time_list: &[i64],
    ) -> Result<Vec<ServiceEventInfo>> {
        let body = self
            .send_cmd(CMD_EPG_SRV_ENUM_PG_INFO_EX, |writer| {
                write_i64_vector(writer, service_time_list)
            })
            .await?;
        let mut reader = Reader::new(&body);
        reader.read_vector(read_service_event_info)
    }

    pub async fn enum_pg_arc(&self, service_time_list: &[i64]) -> Result<Vec<ServiceEventInfo>> {
        let body = self
            .send_cmd(CMD_EPG_SRV_ENUM_PG_ARC, |writer| {
                write_i64_vector(writer, service_time_list)
            })
            .await?;
        let mut reader = Reader::new(&body);
        reader.read_vector(read_service_event_info)
    }

    pub async fn file_copy(&self, name: &str) -> Result<Vec<u8>> {
        self.send_cmd(CMD_EPG_SRV_FILE_COPY, |writer| writer.write_string(name))
            .await
    }

    pub async fn file_copy2(&self, names: &[String]) -> Result<Vec<FileData>> {
        let body = self
            .send_cmd2(CMD_EPG_SRV_FILE_COPY2, |writer| {
                write_string_vector(writer, names)
            })
            .await?;
        read_versioned(&body, |reader| reader.read_vector(read_file_data))
    }

    pub async fn enum_reserve(&self) -> Result<Vec<ReserveData>> {
        let body = self.send_cmd2(CMD_EPG_SRV_ENUM_RESERVE2, |_| {}).await?;
        read_versioned(&body, |reader| reader.read_vector(read_reserve_data))
    }

    pub async fn get_reserve(&self, reserve_id: i32) -> Result<ReserveData> {
        let body = self
            .send_cmd2(CMD_EPG_SRV_GET_RESERVE2, |writer| {
                writer.write_i32(reserve_id)
            })
            .await?;
        read_versioned(&body, read_reserve_data)
    }

    pub async fn get_default_reserve(&self) -> Result<ReserveData> {
        self.get_reserve(DEFAULT_RESERVE_ID).await
    }

    pub async fn add_reserve(&self, reserve: &ReserveData) -> Result<()> {
        self.add_reserves(std::slice::from_ref(reserve)).await
    }

    pub async fn add_reserves(&self, reserves: &[ReserveData]) -> Result<()> {
        self.send_cmd2(CMD_EPG_SRV_ADD_RESERVE2, |writer| {
            writer.write_vector(reserves, write_reserve_data)
        })
        .await?;
        Ok(())
    }

    pub async fn enum_rec_info_basic(&self) -> Result<Vec<RecFileInfo>> {
        let body = self
            .send_cmd2(CMD_EPG_SRV_ENUM_RECINFO_BASIC2, |_| {})
            .await?;
        read_versioned(&body, |reader| reader.read_vector(read_rec_file_info))
    }

    pub async fn get_rec_info(&self, info_id: i32) -> Result<RecFileInfo> {
        let body = self
            .send_cmd2(CMD_EPG_SRV_GET_RECINFO2, |writer| writer.write_i32(info_id))
            .await?;
        read_versioned(&body, read_rec_file_info)
    }

    pub async fn get_rec_file_network_path(&self, path: &str) -> Result<String> {
        let body = self
            .send_cmd(CMD_EPG_SRV_GET_NETWORK_PATH, |writer| {
                writer.write_string(path)
            })
            .await?;
        let mut reader = Reader::new(&body);
        reader.read_string()
    }

    pub async fn get_rec_file_path(&self, reserve_id: i32) -> Result<String> {
        let body = self
            .send_cmd(CMD_EPG_SRV_NWPLAY_TF_OPEN, |writer| {
                writer.write_i32(reserve_id)
            })
            .await?;
        let mut reader = Reader::new(&body);
        let info = read_nw_play_time_shift_info(&mut reader)?;
        let _ = self
            .send_cmd(CMD_EPG_SRV_NWPLAY_CLOSE, |writer| {
                writer.write_i32(info.ctrl_id)
            })
            .await;
        Ok(info.file_path)
    }

    pub async fn enum_tuner_reserve(&self) -> Result<Vec<TunerReserveInfo>> {
        let body = self
            .send_cmd(CMD_EPG_SRV_ENUM_TUNER_RESERVE, |_| {})
            .await?;
        let mut reader = Reader::new(&body);
        reader.read_vector(read_tuner_reserve_info)
    }

    pub async fn enum_tuner_process(&self) -> Result<Vec<TunerProcessStatusInfo>> {
        let body = self
            .send_cmd(CMD_EPG_SRV_ENUM_TUNER_PROCESS, |_| {})
            .await?;
        let mut reader = Reader::new(&body);
        reader.read_vector(read_tuner_process_status_info)
    }

    pub async fn enum_plugin(&self, kind: PluginKind) -> Result<Vec<String>> {
        let body = self
            .send_cmd(CMD_EPG_SRV_ENUM_PLUGIN, |writer| {
                writer.write_u16(kind as u16)
            })
            .await?;
        let mut reader = Reader::new(&body);
        reader.read_vector(|reader| reader.read_string())
    }

    pub async fn search_pg(&self, keys: &[SearchKeyInfo]) -> Result<Vec<EventInfo>> {
        let body = self
            .send_cmd(CMD_EPG_SRV_SEARCH_PG, |writer| {
                writer.write_vector(keys, write_search_key_info)
            })
            .await?;
        let mut reader = Reader::new(&body);
        reader.read_vector(read_event_info)
    }

    pub async fn search_programs(&self, query: &ProgramSearchQuery) -> Result<Vec<EventInfo>> {
        let service_events = self
            .enum_pg_info_ex(&event_lookup_filter(query.service))
            .await?;
        Ok(service_events
            .into_iter()
            .flat_map(|service| service.event_list)
            .filter(|event| event_matches_query(event, query))
            .collect())
    }

    pub async fn preview_reservation(&self, event_key: EventKey) -> Result<ReserveData> {
        let (service, event) = self.find_event(event_key).await?;
        let default = self.get_default_reserve().await?;
        build_reservation_from_event(&default, &service, &event)
    }

    pub async fn create_reservation(&self, event_key: EventKey) -> Result<ReserveData> {
        let reserve = self.preview_reservation(event_key).await?;
        self.add_reserve(&reserve).await?;
        Ok(reserve)
    }

    pub async fn enum_auto_add(&self) -> Result<Vec<AutoAddData>> {
        let body = self.send_cmd2(CMD_EPG_SRV_ENUM_AUTO_ADD2, |_| {}).await?;
        read_versioned(&body, |reader| reader.read_vector(read_auto_add_data))
    }

    pub async fn enum_manual_add(&self) -> Result<Vec<ManualAutoAddData>> {
        let body = self.send_cmd2(CMD_EPG_SRV_ENUM_MANU_ADD2, |_| {}).await?;
        read_versioned(&body, |reader| {
            reader.read_vector(read_manual_auto_add_data)
        })
    }

    pub async fn get_notify_srv_info(&self, target_count: u32) -> Result<NotifySrvInfo> {
        let body = self
            .send_cmd2(CMD_EPG_SRV_GET_STATUS_NOTIFY2, |writer| {
                writer.write_u32(target_count)
            })
            .await?;
        read_versioned(&body, read_notify_srv_info)
    }

    pub async fn get_notify_srv_status(&self) -> Result<NotifySrvInfo> {
        self.get_notify_srv_info(0).await
    }

    async fn send_cmd(&self, cmd: i32, write_payload: impl FnOnce(&mut Writer)) -> Result<Vec<u8>> {
        let mut writer = Writer::new();
        writer.write_i32(cmd);
        writer.write_i32(0);
        write_payload(&mut writer);
        let payload_len = i32::try_from(writer.as_slice().len() - 8)
            .expect("EDCB request payload length fits in i32");
        writer.write_i32_at(4, payload_len);
        self.send_and_receive(writer.into_inner()).await
    }

    async fn send_cmd2(
        &self,
        cmd: i32,
        write_payload: impl FnOnce(&mut Writer),
    ) -> Result<Vec<u8>> {
        self.send_cmd(cmd, |writer| {
            writer.write_u16(CMD_VER);
            write_payload(writer);
        })
        .await
    }

    async fn send_and_receive(&self, request: Vec<u8>) -> Result<Vec<u8>> {
        let addr = format!("{}:{}", self.host, self.port);
        let mut stream = time::timeout(self.timeout, TcpStream::connect(addr))
            .await
            .map_err(|_| EdcbError::Timeout)??;

        time::timeout(self.timeout, stream.write_all(&request))
            .await
            .map_err(|_| EdcbError::Timeout)??;

        let mut header = [0_u8; 8];
        time::timeout(self.timeout, stream.read_exact(&mut header))
            .await
            .map_err(|_| EdcbError::Timeout)??;

        let status = i32::from_le_bytes(header[0..4].try_into().expect("header field length"));
        let size = i32::from_le_bytes(header[4..8].try_into().expect("header field length"));
        let size = usize::try_from(size)
            .map_err(|_| EdcbError::Decode("negative response body size".to_string()))?;
        let mut body = vec![0_u8; size];
        time::timeout(self.timeout, stream.read_exact(&mut body))
            .await
            .map_err(|_| EdcbError::Timeout)??;

        if status == CMD_SUCCESS {
            Ok(body)
        } else {
            Err(EdcbError::CommandStatus(status))
        }
    }

    async fn find_event(&self, event_key: EventKey) -> Result<(ServiceInfo, EventInfo)> {
        let services = self
            .enum_pg_info_ex(&event_lookup_filter(Some(event_key.service)))
            .await?;
        for service in services {
            for event in service.event_list {
                if event.eid == event_key.eid
                    && event.onid == event_key.service.onid
                    && event.tsid == event_key.service.tsid
                    && event.sid == event_key.service.sid
                {
                    return Ok((service.service_info, event));
                }
            }
        }
        Err(EdcbError::InvalidInput(format!(
            "event not found: {}:{}:{}:{}",
            event_key.service.onid, event_key.service.tsid, event_key.service.sid, event_key.eid
        )))
    }
}

fn event_lookup_filter(service: Option<ServiceKey>) -> [i64; 4] {
    let (mask, key) = service
        .map(|service| (0, service.to_search_id()))
        .unwrap_or((EPG_SERVICE_ALL_MASK, EPG_SERVICE_ALL_MASK));
    [mask, key, EPG_LOOKUP_TIME_BEGIN, EPG_LOOKUP_TIME_END]
}

fn event_matches_query(event: &EventInfo, query: &ProgramSearchQuery) -> bool {
    if query.keyword.is_empty() {
        return true;
    }
    let title = event
        .short_info
        .as_ref()
        .map(|info| info.event_name.as_str())
        .unwrap_or("");
    if title.contains(&query.keyword) {
        return true;
    }
    if query.title_only {
        return false;
    }
    event
        .short_info
        .as_ref()
        .is_some_and(|info| info.text_char.contains(&query.keyword))
        || event
            .ext_info
            .as_ref()
            .is_some_and(|info| info.text_char.contains(&query.keyword))
}

pub fn build_reservation_from_event(
    default: &ReserveData,
    service: &ServiceInfo,
    event: &EventInfo,
) -> Result<ReserveData> {
    let start_time = event
        .start_time
        .ok_or_else(|| EdcbError::InvalidInput("event start_time is missing".to_string()))?;
    let duration_second = event
        .duration_sec
        .ok_or_else(|| EdcbError::InvalidInput("event duration_sec is missing".to_string()))?;
    let duration_second = u32::try_from(duration_second).map_err(|_| {
        EdcbError::InvalidInput(format!(
            "event duration_sec must be non-negative: {duration_second}"
        ))
    })?;
    let title = event
        .short_info
        .as_ref()
        .map(|info| info.event_name.trim())
        .filter(|title| !title.is_empty())
        .unwrap_or(default.title.as_str())
        .to_string();

    let mut reserve = default.clone();
    reserve.title = title;
    reserve.start_time = start_time;
    reserve.duration_second = duration_second;
    reserve.station_name = service.service_name.clone();
    reserve.onid = event.onid;
    reserve.tsid = event.tsid;
    reserve.sid = event.sid;
    reserve.eid = event.eid;
    reserve.comment.clear();
    reserve.reserve_id = 0;
    reserve.overlap_mode = 0;
    reserve.start_time_epg = start_time;
    reserve.rec_file_name_list.clear();
    Ok(reserve)
}

fn read_versioned<T>(body: &[u8], read: impl FnOnce(&mut Reader<'_>) -> Result<T>) -> Result<T> {
    let mut reader = Reader::new(body);
    let version = reader.read_u16()?;
    if version < CMD_VER {
        return Err(EdcbError::Decode(format!(
            "unsupported EDCB response version {version}"
        )));
    }
    read(&mut reader)
}

const _: () = {
    let _ = CMD_EPG_SRV_DEL_RESERVE;
    let _ = CMD_EPG_SRV_DEL_RECINFO;
    let _ = CMD_EPG_SRV_CHG_PATH_RECINFO;
    let _ = CMD_EPG_SRV_EPG_CAP_NOW;
    let _ = CMD_EPG_SRV_GET_RESERVE2;
    let _ = CMD_EPG_SRV_ADD_RESERVE2;
    let _ = CMD_EPG_SRV_CHG_RESERVE2;
    let _ = CMD_EPG_SRV_CHG_PROTECT_RECINFO2;
    let _ = CMD_EPG_SRV_ADD_AUTO_ADD2;
    let _ = CMD_EPG_SRV_CHG_AUTO_ADD2;
    let _ = CMD_EPG_SRV_DEL_AUTO_ADD;
    let _ = CMD_EPG_SRV_ADD_MANU_ADD2;
    let _ = CMD_EPG_SRV_CHG_MANU_ADD2;
    let _ = CMD_EPG_SRV_DEL_MANU_ADD;
};
