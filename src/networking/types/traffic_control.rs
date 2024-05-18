use std::{
    collections::HashMap,
    fs::File,
    io,
    process::{Child, Command},
};

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub enum ThrottlingTarget {
    Pid(u32),
    PortEgress(u16),
    PortIngress(u16),
    Interface,
}

#[derive(Debug)]
pub struct TrafficControl {
    identifier_counter: u16,
    identifiers_table: HashMap<ThrottlingTarget, u16>,
    interface: String,
}

pub struct IngressThrottleConfig {
    pub kbps: usize,
    pub burst_kb: usize,
}

impl TrafficControl {
    pub fn new(interface: String, interface_config: Option<IngressThrottleConfig>) -> Self {
        // Clean up all the qdiscs and otherwise before doing anything in case there are previous
        // qdiscs (like if the application was irregularly terminated before and the resources were
        // not properly cleaned)
        let _ = Command::new("tc")
            .args(["qdisc", "del", "dev", &interface, "root"])
            .output();
        // Clean the ingress qdisc
        let _ = Command::new("tc")
            .args(["qdisc", "del", "dev", &interface, "ingress"])
            .output();

        // Create the root qdisc.
        if Command::new("tc")
            .args([
                "qdisc", "add", "dev", &interface, "root", "handle", "1:", "htb",
            ])
            .output()
            .is_err()
        {
            eprintln!("Couldn't make new root qdisc for interface {}", &interface);
        }

        // Create the ingress qdisc
        if Command::new("tc")
            .args(["qdisc", "add", "dev", &interface, "ingress"])
            .output()
            .is_err()
        {
            eprintln!(
                "Couldn't make new ingress qdisc for interface {}",
                &interface
            );
        }

        // If we have a valid interface config, throttle the interface
        if let Some(ingress_config) = interface_config {
            if Command::new("tc")
                .args([
                    "filter",
                    "add",
                    "dev",
                    &interface,
                    "parent",
                    "ffff:",
                    "protocol",
                    "ip",
                    "u32",
                    "match",
                    "u32",
                    "0",
                    "0",
                    "police",
                    "rate",
                    &format!("{}kbps", ingress_config.kbps),
                    "burst",
                    &format!("{}k", ingress_config.burst_kb),
                    "drop",
                    "flowid",
                    ":1",
                ])
                .output()
                .is_err()
            {
                eprintln!(
                    "Couldn't throttle the ingress of the interface {}",
                    &interface
                );
            }
        }

        // Create the filter that redirects cgroup packets to corresponding classes
        let _ = Command::new("tc")
            .args([
                "filter", "add", "dev", &interface, "parent", "1:", "handle", "1:", "cgroup",
            ])
            .output();

        Self {
            identifiers_table: HashMap::new(),
            interface,
            identifier_counter: 1,
        }
    }

    /// Returns true if the connection is throttled, false otherwise
    pub fn pid_is_throttled(&self, pid: u32) -> bool {
        self.identifiers_table
            .get(&ThrottlingTarget::Pid(pid))
            .is_some()
    }

    /// TODO: More testing needed
    ///
    /// Returns true if the port is throttled, false otherwise
    pub fn port_is_throttled(&self, port: u16) -> bool {
        self.identifiers_table
            .get(&ThrottlingTarget::PortEgress(port))
            .is_some()
            && self
                .identifiers_table
                .get(&ThrottlingTarget::PortIngress(port))
                .is_some()
    }

    /// TODO: More testing needed
    ///
    /// If the burst is not specified, the default of 256k is used
    pub fn throttle_port(
        &mut self,
        port: u16,
        kilobytes_per_second: usize,
        burst_in_kilobytes: Option<usize>,
    ) -> io::Result<()> {
        // Get prios for ingress and egress filters. prio will be used as a kind of ID to delete
        // the filters when we need to unthrottle/rethrottle
        let egress_port_prio = self
            .identifiers_table
            .get(&ThrottlingTarget::PortEgress(port))
            .unwrap_or(&self.identifier_counter)
            .to_owned();
        let ingress_port_prio = self
            .identifiers_table
            .get(&ThrottlingTarget::PortIngress(port))
            .unwrap_or(&(&self.identifier_counter + 1))
            .to_owned();

        // Remove any old filter that throttles port on egress
        let _ = Command::new("tc")
            .args([
                "filter",
                "del",
                "dev",
                &self.interface,
                "prio",
                &egress_port_prio.to_string(),
            ])
            .output();

        // Create the filter to throttle the port on egress
        Command::new("tc")
            .args([
                "filter",
                "add",
                "dev",
                &self.interface,
                "parent",
                "1:",
                "protocol",
                "ip",
                "prio",
                &egress_port_prio.to_string(),
                "basic",
                "match",
                "'cmp(u16",
                "at",
                "0",
                "layer",
                "transport",
                "eq",
                &format!("{})'", port),
                "action",
                "police",
                "rate",
                &format!("{}kbps", kilobytes_per_second),
                "burst",
                &format!("{}k", burst_in_kilobytes.unwrap_or(256)),
            ])
            .output()?;

        // Remove any old filter that throttles port on ingress
        let _ = Command::new("tc")
            .args([
                "filter",
                "del",
                "dev",
                &self.interface,
                "prio",
                &ingress_port_prio.to_string(),
            ])
            .output();

        // Create the filter to throttle the port on ingress
        Command::new("tc")
            .args([
                "filter",
                "add",
                "dev",
                &self.interface,
                "ingress",
                "protocol",
                "ip",
                "prio",
                &egress_port_prio.to_string(),
                "basic",
                "match",
                "'cmp(u16",
                "at",
                "2",
                "layer",
                "transport",
                "eq",
                &format!("{})'", port),
                "action",
                "police",
                "rate",
                &format!("{}kbps", kilobytes_per_second),
                "burst",
                &format!("{}k", burst_in_kilobytes.unwrap_or(256)),
            ])
            .output()?;

        // If everything successful, push to `process_to_control_groups`
        self.identifiers_table
            .insert(ThrottlingTarget::PortEgress(port), egress_port_prio);
        self.identifiers_table
            .insert(ThrottlingTarget::PortIngress(port), ingress_port_prio);

        // If we used a new identifier (used the counter), then increment the counter
        if egress_port_prio == self.identifier_counter {
            self.identifier_counter += 1;
        }
        if ingress_port_prio == self.identifier_counter {
            self.identifier_counter += 1;
        }

        Ok(())
    }

    /// TODO: More testing needed
    ///
    /// Unthrottles a port
    pub fn unthrottle_port(&mut self, port: u16) -> io::Result<()> {
        // Remove the filters that are being used to throttle the port
        // Get prios for ingress and egress filters. prio will be used as a kind of ID to delete
        // the filters when we need to unthrottle/rethrottle
        if self.port_is_throttled(port) {
            let egress_port_prio = self
                .identifiers_table
                .get(&ThrottlingTarget::PortEgress(port))
                .unwrap()
                .to_owned();
            let ingress_port_prio = self
                .identifiers_table
                .get(&ThrottlingTarget::PortIngress(port))
                .unwrap()
                .to_owned();

            // Remove any old filter that throttles port on egress
            Command::new("tc")
                .args([
                    "filter",
                    "del",
                    "dev",
                    &self.interface,
                    "prio",
                    &egress_port_prio.to_string(),
                ])
                .output()?;

            // Remove any old filter that throttles port on ingress
            Command::new("tc")
                .args([
                    "filter",
                    "del",
                    "dev",
                    &self.interface,
                    "prio",
                    &ingress_port_prio.to_string(),
                ])
                .output()?;
        }

        Ok(())
    }

    /// This is based on the information obtained from here:
    /// https://unix.stackexchange.com/questions/328308/how-can-i-limit-download-bandwidth-of-an-existing-process-iptables-tc
    /// NOTE: This only throttles on egress, as throttling on ingress by PID proved to be very hard
    pub fn throttle_pid(&mut self, pid: u32, kilobytes_per_second: usize) -> io::Result<()> {
        let pid_classid = self
            .identifiers_table
            .get(&ThrottlingTarget::Pid(pid))
            .unwrap_or(&self.identifier_counter)
            .to_owned();

        if !self.pid_is_throttled(pid) {
            // Create a control group for the pid
            Command::new("cgcreate")
                .args(["-g", &format!("net_cls:sniffnet_{}", pid)])
                .output()
                .unwrap();

            // Set the classid for the newly created cgroup
            let cgroup_classid_file = File::create(format!(
                "/sys/fs/cgroup/net_cls/sniffnet_{}/net_cls.classid",
                pid
            ))
            .unwrap();
            let mut cgroup_classid: Child = Command::new("echo")
                .arg(&format!("0x1{:04x}", pid_classid))
                .stdout(cgroup_classid_file)
                .spawn()
                .unwrap();
            cgroup_classid.wait().unwrap();

            // Move the pid into the newly created cgroup
            Command::new("cgclassify")
                .args(["-g", &format!("net_cls:sniffnet_{}", pid), &pid.to_string()])
                .output()
                .unwrap();
        }

        // Remove the class of the pid if it exists, and don't do anything otherwise. This is
        // useful when we are rethrottling a pid, since in that case the class already exists and
        // we need to remove it first in order to change the throttling speed
        let _ = Command::new("tc")
            .args([
                "class",
                "del",
                "dev",
                &self.interface,
                "classid",
                &format!("1:{}", pid_classid),
            ])
            .output();

        // Add the class with the same classid as the one that was written to the cgroup.
        // The filter added above makes it so that traffic coming from a cgroup with a certain
        // classid will be put in the tc class with the corresponding classid
        Command::new("tc")
            .args([
                "class",
                "add",
                "dev",
                &self.interface,
                "parent",
                "1:",
                "classid",
                &format!("1:{}", pid_classid),
                "htb",
                "rate",
                &format!("{}kbps", kilobytes_per_second),
            ])
            .output()?;

        // If everything successful, push to `process_to_control_groups`
        self.identifiers_table
            .insert(ThrottlingTarget::Pid(pid), pid_classid);

        // If we used a new classid (used the counter), then increment the counter
        if pid_classid == self.identifier_counter {
            self.identifier_counter += 1;
        }

        Ok(())
    }

    pub fn unthrottle_pid(&mut self, pid: u32) -> io::Result<()> {
        // TODO: also remove pid from cgroup
        // For now, just removing the tc class will suffice, since that is what is being used to
        // set the actual throttling

        // Remove the class of the pid if it exists
        if self.pid_is_throttled(pid) {
            Command::new("tc")
                .args([
                    "class",
                    "del",
                    "dev",
                    &self.interface,
                    "classid",
                    &format!(
                        "1:{}",
                        self.identifiers_table
                            .remove(&ThrottlingTarget::Pid(pid))
                            .unwrap_or_else(|| panic!("Couldn't unthrottle {}", pid))
                    ),
                ])
                .output()?;
        }

        Ok(())
    }

    // TODO: Also run this function when we return from a scan and change interfaces
    pub fn clean_traffic_control_settings(interface: String) {
        Command::new("tc")
            .args(["qdisc", "del", "dev", &interface, "root"])
            .output()
            .expect(&("root qdisc couldn't be freed for the interface".to_string() + &interface));

        // Clean the ingress qdisc
        Command::new("tc")
            .args(["qdisc", "del", "dev", &interface, "ingress"])
            .output()
            .expect(
                &("ingress qdisc couldn't be freed for the interface".to_string() + &interface),
            );
    }
}

impl Drop for TrafficControl {
    // Clean up the qdiscs and classes and filters...
    // TODO: also move all the processes out of the cgroups they are in
    // TODO: run this when application terminates to ensure the cleaning up happens in that case
    fn drop(&mut self) {
        // Command::new("tc")
        //     .args(["qdisc", "del", "dev", &self.interface, "root"])
        //     .output()
        //     .expect(
        //         &("root qdisc couldn't be freed for the interface".to_string() + &self.interface),
        //     );

        // // Clean the ingress qdisc
        // Command::new("tc")
        //     .args(["qdisc", "del", "dev", &self.interface, "ingress"])
        //     .output()
        //     .expect(
        //         &("ingress qdisc couldn't be freed for the interface".to_string()
        //             + &self.interface),
        //     );
    }
}
