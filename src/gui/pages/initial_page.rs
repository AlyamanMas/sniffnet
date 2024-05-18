//! Module defining the initial page of the application.
//!
//! It contains elements to select network adapter and traffic filters.

use iced::advanced::renderer::Style;
use iced::advanced::widget::Text;
use iced::theme::palette::Background;
use iced::widget::scrollable::Direction;
use iced::widget::tooltip::Position;
use iced::widget::{
    button, horizontal_space, vertical_space, Button, Column, Container, PickList, Row, Scrollable,
    Tooltip, TextInput
};
use iced::Color;
use iced::Renderer;
use iced::widget::text_input::StyleSheet;
use iced::Length::FillPortion;
use iced::{alignment, Alignment, Font, Length};
use pcap::Device;

use crate::gui::components::radio::{ip_version_radios, transport_protocol_radios};
use crate::gui::styles::button::{ButtonStyleTuple, ButtonType};
use crate::gui::styles::container::{ContainerStyleTuple, ContainerType};
use crate::gui::styles::picklist::{PicklistStyleTuple, PicklistType};
use crate::gui::styles::scrollbar::{ScrollbarStyleTuple, ScrollbarType};
use crate::gui::styles::style_constants::{get_font, FONT_SIZE_SUBTITLE, FONT_SIZE_TITLE, ICONS};
use crate::gui::styles::text::{TextStyleTuple, TextType};
use crate::gui::styles::types::gradient_type::GradientType;
use crate::gui::types::message::Message;
use crate::gui::types::sniffer::Sniffer;
use crate::translations::translations::{
    address_translation, addresses_translation, all_translation, application_protocol_translation,
    choose_adapters_translation, select_filters_translation, start_translation,
};
use crate::{AppProtocol, Language, StyleType};

/// Computes the body of gui initial page
pub fn initial_page(sniffer: &Sniffer) -> Container<Message> {
    let font = get_font(sniffer.style);

    let col_adapter = get_col_adapter(sniffer, font);

    let pid_textbox: TextInput<Message,Renderer> = TextInput::new(
        "",
        &sniffer.filters.pid,
    ).width(60).on_input(Message::PidFilter);
    let pid_filter = Column::new(
        ).push(
            Text::new("PID")
                .font(font)
                .style(TextStyleTuple(sniffer.style, TextType::Subtitle))
                .size(FONT_SIZE_SUBTITLE),
        ).push(pid_textbox);


    let uid_textbox: TextInput<Message,Renderer> = TextInput::new(
        "",
        &sniffer.filters.uid,
    ).width(60).on_input(Message::UidFilter);

    let uid_filter = Column::new(
        ).push(
            Text::new("UID")
                .font(font)
                .style(TextStyleTuple(sniffer.style, TextType::Subtitle))
                .size(FONT_SIZE_SUBTITLE),
        ).push(uid_textbox);
    let port_textbox: TextInput<Message,Renderer> = TextInput::new(
        "",
        &sniffer.filters.port,
    ).width(60).on_input(Message::PortFilter);

    let port_filter = Column::new(
        ).push(
            Text::new("Port")
                .font(font)
                .style(TextStyleTuple(sniffer.style, TextType::Subtitle))
                .size(FONT_SIZE_SUBTITLE),
        ).push(port_textbox);
    
    let ip_active = sniffer.filters.ip;
    let col_ip_radio = ip_version_radios(ip_active, font, sniffer.style, sniffer.language);
    let col_ip = Column::new()
        .spacing(10)
        .width(FillPortion(5))
        .push(col_ip_radio).push(
            Row::new()
                .push(pid_filter)
                .push(uid_filter)
                .push(port_filter
        ).spacing(10).align_items(Alignment::Center));

    let transport_active = sniffer.filters.transport;
    let col_transport_radio =
        transport_protocol_radios(transport_active, font, sniffer.style, sniffer.language);
    let col_transport = Column::new()
        .align_items(Alignment::Center)
        .spacing(10)
        .width(FillPortion(9))
        .push(col_transport_radio);

    let app_active = if sniffer.filters.application.ne(&AppProtocol::Other) {
        Some(sniffer.filters.application)
    } else {
        None
    };
    let picklist_app = PickList::new(
        if app_active.is_some() {
            &AppProtocol::ALL[..]
        } else {
            &AppProtocol::ALL[1..]
        },
        app_active,
        Message::AppProtocolSelection,
    )
    .padding([3, 7])
    .placeholder(all_translation(sniffer.language))
    .font(font)
    .style(PicklistStyleTuple(sniffer.style, PicklistType::Standard));
    let col_app = Column::new()
        .width(FillPortion(8))
        .spacing(10)
        .push(
            Text::new(application_protocol_translation(sniffer.language))
                .font(font)
                .style(TextStyleTuple(sniffer.style, TextType::Subtitle))
                .size(FONT_SIZE_SUBTITLE),
        )
        .push(picklist_app);

    

    let filters = Column::new()
        .width(FillPortion(6))
        .padding(10)
        .spacing(5)
        .push(
            Row::new().push(
                select_filters_translation(sniffer.language)
                    .font(font)
                    .style(TextStyleTuple(sniffer.style, TextType::Title))
                    .size(FONT_SIZE_TITLE),
            ),
        )
        .push(
            Row::new()
                .spacing(10)
                .height(FillPortion(3))
                .push(col_ip)
                .push(col_transport)
                .push(col_app),
        ).push(
            Row::new()
                .push(
                    button_start(sniffer.style, sniffer.language, sniffer.color_gradient)
                        
                )
                .align_items(Alignment::Center),
        );
    // add a UI element to specify the percentage of throttlling to apply on the selected network adapter
    let throttling_textbox: TextInput<Message,Renderer> = TextInput::new(
        "",
        // if bandwidth id umax, leave the textbox empty, otherwise show the value
        // not working 
        &sniffer.interface_bandwidth,
    ).width(60).on_input(Message::InterfaceBandwidth);
    let throttling_interface = Column::new(
        ).push(
            Text::new("Throttling")
                .font(font)
                .style(TextStyleTuple(sniffer.style, TextType::Subtitle))
                .size(FONT_SIZE_SUBTITLE),
        ).push(throttling_textbox);
    
    let body = Column::new().push(vertical_space(Length::Fixed(5.0))).push(
        Row::new()
            .push(col_adapter)
            .push(throttling_interface).align_items(Alignment::Center)
            // add a rectange with maximum height of 1 pixel to separate the adapter list from the filters
            .push(Container::new(Text::new("")).width(Length::Fixed(10.0)).height(Length::Fill))
            .push(filters),
    );

    Container::new(body)
        .height(Length::Fill)
        .style(<ContainerStyleTuple as Into<iced::theme::Container>>::into(
            ContainerStyleTuple(sniffer.style, ContainerType::Standard),
        ))
}

fn button_start(
    style: StyleType,
    language: Language,
    color_gradient: GradientType,
) -> Tooltip<'static, Message> {
    let content = button(
        Text::new("S")
            .font(ICONS)
            .size(25)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center),
    )
    .padding(10)
    .height(Length::Fixed(80.0))
    .width(Length::Fixed(160.0))
    .style(ButtonStyleTuple(style, ButtonType::Gradient(color_gradient)).into())
    .on_press(Message::Start);

    let tooltip = start_translation(language).to_string();
    //tooltip.push_str(" [⏎]");
    Tooltip::new(content, tooltip, Position::Top)
        .gap(5)
        .font(get_font(style))
        .style(<ContainerStyleTuple as Into<iced::theme::Container>>::into(
            ContainerStyleTuple(style, ContainerType::Tooltip),
        ))
}

fn get_col_adapter(sniffer: &Sniffer, font: Font) -> Column<Message> {
    let mut dev_str_list = vec![];
    for dev in Device::list().expect("Error retrieving device list\r\n") {
        let mut dev_str = String::new();
        let name = dev.name;
        match dev.desc {
            None => {
                dev_str.push_str(&name);
            }
            Some(description) => {
                #[cfg(not(target_os = "windows"))]
                dev_str.push_str(&format!("{name}\n"));
                dev_str.push_str(&description);
            }
        }
        let num_addresses = dev.addresses.len();
        match num_addresses {
            0 => {}
            1 => {
                dev_str.push_str(&format!("\n{}:", address_translation(sniffer.language)));
            }
            _ => {
                dev_str.push_str(&format!("\n{}:", addresses_translation(sniffer.language)));
            }
        }

        for addr in dev.addresses {
            let address_string = addr.addr.to_string();
            dev_str.push_str(&format!("\n   {address_string}"));
        }
        dev_str_list.push((name, dev_str));
    }

    Column::new()
        .padding(10)
        .spacing(5)
        .height(Length::Fill)
        .width(FillPortion(4))
        .push(
            choose_adapters_translation(sniffer.language)
                .font(font)
                .style(TextStyleTuple(sniffer.style, TextType::Title))
                .size(FONT_SIZE_TITLE),
        )
        .push(
            Scrollable::new(dev_str_list.iter().fold(
                Column::new().padding(13).spacing(5),
                |scroll_adapters, adapter| {
                    let name = adapter.0.clone();
                    let description = adapter.1.clone();
                    scroll_adapters.push(
                        Button::new(Text::new(description).font(font))
                            .padding([20, 30])
                            .width(Length::Fill)
                            .style(
                                ButtonStyleTuple(
                                    sniffer.style,
                                    if name == sniffer.device.name {
                                        ButtonType::BorderedRoundSelected
                                    } else {
                                        ButtonType::BorderedRound
                                    },
                                )
                                .into(),
                            )
                            .on_press(Message::AdapterSelection(name)),
                    )
                },
            ))
            .direction(Direction::Vertical(ScrollbarType::properties()))
            .style(
                <ScrollbarStyleTuple as Into<iced::theme::Scrollable>>::into(ScrollbarStyleTuple(
                    sniffer.style,
                    ScrollbarType::Standard,
                )),
            ),
        )
}
