use std::net::IpAddr;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::tooltip::Position;
use iced::widget::{button, horizontal_space, lazy, vertical_space, Rule};
use iced::widget::{Column, Container, Row, Text, Tooltip};
use iced::Length::Fixed;
use iced::{Alignment, Font, Length};

use crate::countries::country_utils::{get_computer_tooltip, get_flag_tooltip};
use crate::countries::flags_pictures::FLAGS_WIDTH_BIG;
use crate::gui::styles::button::{ButtonStyleTuple, ButtonType};
use crate::gui::styles::container::{ContainerStyleTuple, ContainerType};
use crate::gui::styles::rule::{RuleStyleTuple, RuleType};
use crate::gui::styles::style_constants::{get_font, get_font_headers, FONT_SIZE_TITLE, ICONS};
use crate::gui::styles::text::{TextStyleTuple, TextType};
use crate::gui::styles::types::gradient_type::GradientType;
use crate::gui::types::message::Message;
use crate::networking::manage_packets::{get_address_to_lookup, get_traffic_type, is_my_address};
use crate::networking::types::address_port_pair::AddressPortPair;
use crate::networking::types::host::Host;
use crate::networking::types::info_address_port_pair::InfoAddressPortPair;
use crate::networking::types::traffic_direction::TrafficDirection;
use crate::translations::translations::{
    application_protocol_translation, hide_translation, incoming_translation, outgoing_translation,
    packets_translation, transport_protocol_translation,
};
use crate::translations::translations_2::{
    administrative_entity_translation, connection_details_translation, destination_translation,
    fqdn_translation, mac_address_translation, socket_address_translation, source_translation,
    transmitted_data_translation,
};

use iced::widget::{
    Checkbox, Scrollable,
    TextInput,
};

use crate::gui::styles::text_input::{TextInputStyleTuple, TextInputType};


use crate::utils::formatted_strings::{get_formatted_bytes_string_with_b, get_socket_address};
use crate::{Language, Sniffer, StyleType};


// pub fn process_throttling_page(sniffer: &Sniffer, process_id: u32) -> Container<Message>  {
//     //display hellow world for debugging
//     let font = get_font(sniffer.style);
//     //declare styel of type StyleType
//     let style = sniffer.style;
//     //convert process_id to str and store it in pid_str
//     let pid_str = process_id.to_string();
//     let string = "Please Enter Throttling Value for pid: ".to_string() ;
//     let mut column = Column::new()
//         .spacing(10)
//         .align_items(Alignment::Center)
//         .push(Text::new( string + &pid_str).size(20))
//         .push(TextInput::new("Throttling value", "")
//             // .on_input(move |value| {
//             //     println!("Input value: {}", value);
//             // })
//             .padding([10, 10, 10, 10])
//             .font(font)
//             .width(Length::Fixed(100.0))
//             .style(<TextInputStyleTuple as Into<iced::theme::TextInput>>::into(
//                 TextInputStyleTuple(style, TextInputType::Standard),
//             )),
//         );
//     // column = column.push(Text::new("Throttling Page").size(30));
//     Container::new(column)
//         .style(<ContainerStyleTuple as Into<iced::theme::Container>>::into(
//             ContainerStyleTuple(sniffer.style, ContainerType::Standard),
//         ))

// }

pub fn process_throttling_page(sniffer: &Sniffer, process_id: u32) -> Container<Message> {
    // Display "Hello World" for debugging
    let font = get_font(sniffer.style);
    let style = sniffer.style;
    let pid_str = process_id.to_string();
    let string = "Please Enter Throttling Value for pid: ".to_string();
    
    let column = Column::new()
        .spacing(10)
        .align_items(Alignment::Center)
        .push(Text::new(string + &pid_str).size(20))
        .push(TextInput::new("Throttling value", "")
            .on_input(Message::InputChanged) // Send InputChanged message on input
            .padding([10, 10, 10, 10])
            .font(font)
            .width(Length::Fixed(100.0))
            .style(<TextInputStyleTuple as Into<iced::theme::TextInput>>::into(
                TextInputStyleTuple(style, TextInputType::Standard),
            ))
        );
    
    Container::new(column)
        .style(<ContainerStyleTuple as Into<iced::theme::Container>>::into(
            ContainerStyleTuple(sniffer.style, ContainerType::Standard),
        ))
}