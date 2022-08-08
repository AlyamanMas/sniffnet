mod address_port;
mod report_info;

use std::collections::HashMap;
use std::collections::HashSet;
use etherparse::{IpHeader, PacketHeaders, TransportHeader};
use pcap::{Device, Capture};
use std::fs::File;
use std::io::Write;
use crate::address_port::{AddressPort};
use crate::report_info::{ReportInfo, TransProtocol};
use chrono::prelude::*;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Name of the network adapter to be inspected, if omitted a default adapter is chosen
    #[clap(short, long, value_parser, forbid_empty_values = true, default_value = "en0")]
    adapter: String,

    /// Name of output file to contain the textual report, if omitted a default file is chosen
    #[clap(short, long, value_parser, forbid_empty_values = true, default_value = "report.txt")]
    output_file: String,

    /// Set the minimum port value to be considered, if omitted there is not ports lower bound
    #[clap(short, long, value_parser, default_value_t = u16::MIN)]
    lowest_port: u16,

    /// Set the maximum port value to be considered, if omitted there is not ports higher bound
    #[clap(short, long, value_parser, default_value_t = u16::MAX)]
    highest_port: u16,
}


fn main() {

    let args = Args::parse();
    let adapter: String = args.adapter;
    let output_file: String = args.output_file;
    let lowest_port = args.lowest_port;
    let highest_port = args.highest_port;

    let mut output = File::create(output_file.clone()).unwrap();

    println!("Waiting for packets (sniffing adapter {})........", adapter.clone());
    println!("Considering port numbers from {} to {}", lowest_port, highest_port);
    println!("Writing {} file........", output_file.clone());

    write!(output, "Packets sniffed from adapter '{}'\n\n", adapter.clone());
    write!(output, "Considering port numbers from {} to {}\n\n", lowest_port, highest_port);

    let dev_list = Device::list().expect("Unable to retrieve network adapters list");
    let mut found_device = Device {
        name: "".to_string(),
        desc: None,
        addresses: vec![]
    };
    for device in dev_list {
        if device.name == adapter {
            found_device = device;
            break;
        }
    }
    if found_device.name.len() == 0 {
        panic!("Specified network adapter does not exist");
    }

    let mut cap = Capture::from_device(found_device).unwrap()
        .promisc(true)
        .open().unwrap();
    
    let mut map:HashMap<AddressPort,ReportInfo> = HashMap::new();

    let mut num_packets = 0; //dopo 300 pacchetti interrompo la cattura e stampo
    while let Ok(packet) = cap.next() {

        let utc: DateTime<Local> = Local::now();
        let now = utc.format("%d/%m/%Y %H:%M:%S").to_string();

        match PacketHeaders::from_ethernet_slice(&packet) {
            Err(value) => println!("Err {:?}", value),
            Ok(value) => {

                let address1;
                let address2;
                let mut port1= 0;
                let mut port2= 0;
                let exchanged_bytes: u32;
                let mut protocol = TransProtocol::Other;

                match value.ip.unwrap() {
                    IpHeader::Version4(ipv4header, _) => {
                        address1 = format!("{:?}", ipv4header.source)
                            .replace("[","")
                            .replace("]","")
                            .replace(",",".")
                            .replace(" ","");
                        address2 = format!("{:?}", ipv4header.destination)
                            .replace("[","")
                            .replace("]","")
                            .replace(",",".")
                            .replace(" ","");
                        exchanged_bytes = ipv4header.payload_len as u32;
                    }
                    IpHeader::Version6(ipv6header, _) => {
                        address1 = format!("{:?}", ipv6header.source)
                            .replace("[", "")
                            .replace("]", "")
                            .replace(",", ".")
                            .replace(" ", "");
                        address2 = format!("{:?}", ipv6header.destination)
                            .replace("[", "")
                            .replace("]", "")
                            .replace(",", ".")
                            .replace(" ", "");
                        exchanged_bytes = ipv6header.payload_length as u32;
                    }
                }

                match value.transport.unwrap() {
                    TransportHeader::Udp(udpheader) => {
                        port1 = udpheader.source_port;
                        protocol = TransProtocol::UDP;
                        port2 = udpheader.destination_port
                    }
                    TransportHeader::Tcp(tcpheader) => {
                        port1 = tcpheader.source_port;
                        protocol = TransProtocol::TCP;
                        port2 = tcpheader.destination_port
                    }
                    TransportHeader::Icmpv4(_) => {}
                    TransportHeader::Icmpv6(_) => {}
                }
                
                let key1: AddressPort = AddressPort::new(address1,port1);
                let key2: AddressPort = AddressPort::new(address2,port2);

                if port1 >= lowest_port && port1 <= highest_port {
                    map.entry(key1).and_modify(|info| {
                        info.transmitted_bytes += exchanged_bytes;
                        info.transmitted_packets += 1;
                        info.final_timestamp = now.clone();
                        info.trans_protocols.insert(protocol);})
                        .or_insert(ReportInfo {
                            transmitted_bytes: exchanged_bytes,
                            transmitted_packets: 1,
                            received_bytes: 0,
                            received_packets: 0,
                            initial_timestamp: now.clone(),
                            final_timestamp: now.clone(),
                            trans_protocols: HashSet::from([protocol])
                        });
                }

                if port2 >= lowest_port && port2 <= highest_port {
                    map.entry(key2).and_modify(|info| {
                        info.received_bytes += exchanged_bytes;
                        info.received_packets += 1;
                        info.final_timestamp = now.clone();
                        info.trans_protocols.insert(protocol); })
                        .or_insert(ReportInfo {
                            transmitted_bytes: 0,
                            transmitted_packets: 0,
                            received_bytes: exchanged_bytes,
                            received_packets: 1,
                            initial_timestamp: now.clone(),
                            final_timestamp: now.clone(),
                            trans_protocols: HashSet::from([protocol])
                        });
                }

            }
        }

        num_packets+=1;
        if num_packets >= 300 {
            break;
        }
    }

    for (key, val) in map.iter() {
        write!(output, "Address: {}:{}\n{}\n\n", key.address1, key.port1, val).expect("File output error");
    }
}