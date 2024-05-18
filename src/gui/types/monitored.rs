
// monitored process 
pub struct MonitoredProcess {
    pub pid: u32,
    pub total_sent_bytes: u128,
    pub total_sent_packets: u128,
    pub total_received_bytes: u128,
    pub total_received_packets: u128,
}

impl MonitoredProcess {
    pub fn new(pid: u32) -> Self {
        MonitoredProcess {
            pid,
            total_sent_bytes: 0,
            total_sent_packets: 0,
            total_received_bytes: 0,
            total_received_packets: 0,
        }
    }
}

// monitored port 
pub struct MonitoredPort {
    pub port: u16,
    pub total_sent_bytes: u128,
    pub total_sent_packets: u128,
    pub total_received_bytes: u128,
    pub total_received_packets: u128,
}

impl MonitoredPort {
    pub fn new(port: u16) -> Self {
        MonitoredPort {
            port,
            total_sent_bytes: 0,
            total_sent_packets: 0,
            total_received_bytes: 0,
            total_received_packets: 0,
        }
    }
}

// monitored user

pub struct MonitoredUser {
    pub uid: u32,
    pub total_sent_bytes: u128,
    pub total_sent_packets: u128,
    pub total_received_bytes: u128,
    pub total_received_packets: u128,
}

impl MonitoredUser {
    pub fn new(uid: u32) -> Self {
        MonitoredUser {
            uid,
            total_sent_bytes: 0,
            total_sent_packets: 0,
            total_received_bytes: 0,
            total_received_packets: 0,
        }
    }
}