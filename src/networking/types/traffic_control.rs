use std::{
    collections::HashMap,
    fs::File,
    io,
    process::{Child, Command},
};

pub struct TrafficControl {
    classid_counter: u16,
    process_to_control_groups: HashMap<u32, u16>,
    interface: String,
}

impl TrafficControl {
    pub fn new(interface: String) -> Self {
        // Clean up all the qdiscs and otherwise before doing anything in case there are previous
        // qdiscs (like if the application was irregularly terminated before and the resources were
        // not properly cleaned)
        let _ = Command::new("tc")
            .args(["qdisc", "del", "dev", &interface, "root"])
            .output();
        // .expect(&("root qdisc couldn't be freed for the interface".to_string() + &interface));

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

        // Create the filter that redirects cgroup packets to corresponding classes
        let _ = Command::new("tc")
            .args([
                "filter", "add", "dev", &interface, "parent", "1:", "handle", "1:", "cgroup",
            ])
            .output();

        Self {
            process_to_control_groups: HashMap::new(),
            interface,
            classid_counter: 1,
        }
    }

    /// Returns true if the connection is throttled, false otherwise
    pub fn pid_is_throttled(&self, pid: u32) -> bool {
        self.process_to_control_groups.get(&pid).is_some()
    }

    /// This is based on the information obtained from here:
    /// https://unix.stackexchange.com/questions/328308/how-can-i-limit-download-bandwidth-of-an-existing-process-iptables-tc
    pub fn throttle_pid(&mut self, pid: u32, kilobytes_per_second: usize) -> io::Result<()> {
        let pid_classid = self
            .process_to_control_groups
            .get(&pid)
            .unwrap_or(&self.classid_counter)
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
        self.process_to_control_groups.insert(pid, pid_classid);

        // If we used a new classid (used the counter), then increment the counter
        if pid_classid == self.classid_counter {
            self.classid_counter += 1;
        }

        Ok(())
    }

    pub fn unthrottle(&mut self, pid: u32) -> io::Result<()> {
        // TODO: also remove pid from cgroup
        // For now, just removing the tc class will suffice, since that is what is being used to
        // set the actual throttling

        // Remove the class of the pid if it exists, and don't do anything otherwise. This is
        // useful when we are rethrottling a pid, since in that case the class already exists and
        // we need to remove it first in order to change the throttling speed
        if self.pid_is_throttled(pid) {
            let _ = Command::new("tc")
                .args([
                    "class",
                    "del",
                    "dev",
                    &self.interface,
                    "classid",
                    &format!(
                        "1:{}",
                        self.process_to_control_groups
                            .remove(&pid)
                            .unwrap_or_else(|| panic!("Couldn't unthrottle {}", pid))
                    ),
                ])
                .output();
        }

        Ok(())
    }
}

impl Drop for TrafficControl {
    // Clean up the qdiscs and classes and filters...
    // TODO: also move all the processes out of the cgroups they are in
    // TODO: run this when application terminates to ensure the cleaning up happens in that case
    fn drop(&mut self) {
        Command::new("tc")
            .args(["qdisc", "del", "dev", &self.interface, "root"])
            .output()
            .expect(
                &("root qdisc couldn't be freed for the interface".to_string() + &self.interface),
            );
    }
}
