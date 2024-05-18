use iced::alignment::{Horizontal, Vertical};
use iced::widget::scrollable::Direction;
use iced::widget::tooltip::Position;
use iced::widget::{button, horizontal_space, vertical_space, Rule};
use iced::widget::{
    lazy, Button, Checkbox, Column, Container, PickList, Row, Scrollable, Text, TextInput, Tooltip,
};
// import Radio
use iced::widget::Radio;
use iced::widget::radio::Appearance;
// use crate::gui::components::custom_radio::CustomRadio;
use iced::{alignment, Alignment, Font, Length, Renderer};

use std::collections::HashMap;
use std::u32::MAX;

use crate::gui::components::radio::report_view_radios;
use crate::gui::components::tab::get_pages_tabs;
use crate::gui::components::types::my_modal::MyModal;
use crate::gui::components::types::report_view::ReportView;
use crate::gui::styles::button::{ButtonStyleTuple, ButtonType};
use crate::gui::styles::checkbox::{CheckboxStyleTuple, CheckboxType};
use crate::gui::styles::container::{ContainerStyleTuple, ContainerType};
use crate::gui::styles::picklist::{PicklistStyleTuple, PicklistType};
use crate::gui::styles::radio::{self, RadioStyleTuple};
use crate::gui::styles::rule::{RuleStyleTuple, RuleType};
use crate::gui::styles::scrollbar::{ScrollbarStyleTuple, ScrollbarType};
use crate::gui::styles::style_constants::{get_font, FONT_SIZE_TITLE, ICONS};
use crate::gui::styles::text::{TextStyleTuple, TextType};
use crate::gui::styles::text_input::{TextInputStyleTuple, TextInputType};
use crate::gui::types::message::Message;
use crate::networking::types::search_parameters::{FilterInputType, SearchParameters};
use crate::networking::types::traffic_direction::TrafficDirection;
use crate::networking::types::trans_protocol::TransProtocol;
use crate::report::get_report_entries::get_searched_entries;
use crate::translations::translations::application_protocol_translation;
use crate::translations::translations_2::{
    administrative_entity_translation, country_translation, domain_name_translation,
    no_search_results_translation, only_show_favorites_translation, search_filters_translation,
    showing_results_translation, sort_by_translation,
};
use crate::utils::formatted_strings::get_formatted_bytes_string;
use crate::utils::formatted_strings::{get_connection_color, get_open_report_tooltip};
use crate::{Language, ReportSortType, RunningPage, Sniffer, StyleType};

use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};

/// Computes the body of gui inspect page

pub fn inspect_page(sniffer: &Sniffer) -> Container<Message> {
    let font = get_font(sniffer.style);

    let mut body = Column::new()
        .width(Length::Fill)
        .padding(10)
        .spacing(10)
        .align_items(Alignment::Center);

    let mut tab_and_body = Column::new().height(Length::Fill);

    let tabs = get_pages_tabs(
        [
            RunningPage::Overview,
            RunningPage::Inspect,
            RunningPage::Notifications,
        ],
        &["d ", "5 ", "7 "],
        &[
            Message::ChangeRunningPage(RunningPage::Overview),
            Message::TickInit,
            Message::ChangeRunningPage(RunningPage::Notifications),
        ],
        RunningPage::Inspect,
        sniffer.style,
        sniffer.language,
        sniffer.unread_notifications,
    );

    tab_and_body = tab_and_body.push(tabs);

    let sort_active_str = sniffer
        .report_sort_type
        .get_picklist_label(sniffer.language);
    let sort_list_str: Vec<&str> = ReportSortType::all_strings(sniffer.language);
    let picklist_sort = PickList::new(
        sort_list_str.clone(),
        Some(sort_active_str),
        move |selected_str| {
            if selected_str == *sort_list_str.first().unwrap_or(&"") {
                Message::ReportSortSelection(ReportSortType::MostRecent)
            } else if selected_str == *sort_list_str.get(1).unwrap_or(&"") {
                Message::ReportSortSelection(ReportSortType::MostBytes)
            } else {
                Message::ReportSortSelection(ReportSortType::MostPackets)
            }
        },
    )
    .padding([3, 7])
    .font(font)
    .style(PicklistStyleTuple(sniffer.style, PicklistType::Standard));

    let report = lazy(
        (
            sniffer.runtime_data.tot_sent_packets + sniffer.runtime_data.tot_received_packets,
            &sniffer.style,
            &sniffer.language,
            sniffer.report_sort_type,
            sniffer.search.clone(),
            sniffer.page_number,
        ),
        move |_| lazy_report(sniffer, 
            sniffer.report_view),
    );

    // use radio buttons defined in radio module
    let views_radio_buttons = report_view_radios(
        sniffer.report_view,
        font,
        sniffer.style,
        sniffer.language
    );


    // dbg!(sn);

    body = body
        .push(
            Container::new(
                Row::new()
                    .push(filters_col(
                        &sniffer.search,
                        sniffer.style,
                        sniffer.language,
                    ))
                    .push(Rule::vertical(25).style(
                        <RuleStyleTuple as Into<iced::theme::Rule>>::into(RuleStyleTuple(
                            sniffer.style,
                            RuleType::Standard,
                        )),
                    ))
                    .push(
                        Column::new()
                            .spacing(10)
                            .push(
                                Text::new(sort_by_translation(sniffer.language))
                                    .font(font)
                                    .style(TextStyleTuple(sniffer.style, TextType::Title))
                                    .size(FONT_SIZE_TITLE),
                            )
                            .push(picklist_sort),
                    ),
            )
            .height(Length::Fixed(165.0))
            .padding(10)
            .style(<ContainerStyleTuple as Into<iced::theme::Container>>::into(
                ContainerStyleTuple(sniffer.style, ContainerType::BorderedRound),
            )),
        )
        .push(views_radio_buttons)
        .push(report);

    Container::new(Column::new().push(tab_and_body.push(body)))
        .height(Length::Fill)
        .style(<ContainerStyleTuple as Into<iced::theme::Container>>::into(
            ContainerStyleTuple(sniffer.style, ContainerType::Standard),
        ))
}

fn lazy_report(sniffer: &Sniffer, report_view: ReportView) -> Row<'static, Message> {
    let font = get_font(sniffer.style);

    let (search_results, results_number) = get_searched_entries(sniffer);

    let mut col_report = Column::new()
        .height(Length::Fill)
        .width(Length::Fill)
        .align_items(Alignment::Center);

    match report_view {
        ReportView::Detailed => {
            let mut scroll_report = Column::new();
            let start_entry_num = (sniffer.page_number - 1) * 20 + 1;
            let end_entry_num = start_entry_num + search_results.len() - 1;
            for (key, val, flag) in search_results {
                let entry_color = get_connection_color(val.traffic_direction, sniffer.style);
                let entry_row = Row::new()
                    .align_items(Alignment::Center)
                    .push(
                        Text::new(format!(
                            "          {}{}",
                            key.print_gui(),
                            val.print_gui(),
                        ))
                        .style(iced::theme::Text::Color(entry_color))
                        .font(font),
                    );

                scroll_report = scroll_report.push(
                    button(entry_row)
                        .padding(2)
                        .on_press(Message::ShowModal(MyModal::ConnectionDetails(val.index)))
                        .style(ButtonStyleTuple(sniffer.style, ButtonType::Neutral).into()),
                );
            }
            if results_number > 0 {
                col_report = col_report
                    .push(Text::new("      Src IP address       Src port      Dst IP address       Dst port  Layer4   Layer7     Packets     Bytes    pid     uid").vertical_alignment(Vertical::Center).height(Length::FillPortion(2)).font(font))
                    .push(Rule::horizontal(5).style(<RuleStyleTuple as Into<iced::theme::Rule>>::into(RuleStyleTuple(
                        sniffer.style,
                        RuleType::Standard,
                    ))))
                    .push(
                        Scrollable::new(scroll_report)
                            .height(Length::FillPortion(15))
                            .width(Length::Fill)
                            .direction(Direction::Both {
                                vertical: ScrollbarType::properties(),
                                horizontal: ScrollbarType::properties(),
                            })
                            .style(
                                <ScrollbarStyleTuple as Into<iced::theme::Scrollable>>::into(
                                    ScrollbarStyleTuple(sniffer.style, ScrollbarType::Standard),
                                ),
                            ),
                    )
                    .push(
                        Rule::horizontal(5).style(<RuleStyleTuple as Into<iced::theme::Rule>>::into(
                            RuleStyleTuple(sniffer.style, RuleType::Standard),
                        )),
                    )
                    .push(get_change_page_row(
                        sniffer.style,
                        sniffer.language,
                        sniffer.page_number,
                        start_entry_num,
                        end_entry_num,
                        results_number,
                    ));
            } else {
                col_report = col_report.push(
                    Column::new()
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(20)
                        .align_items(Alignment::Center)
                        .push(vertical_space(Length::FillPortion(1)))
                        .push(Text::new('V'.to_string()).font(ICONS).size(60))
                        .push(vertical_space(Length::Fixed(15.0)))
                        .push(Text::new(no_search_results_translation(sniffer.language)).font(font))
                        .push(vertical_space(Length::FillPortion(2))),
                );
            }
        }
        ReportView::Process => {
            // loop over all search results, and get pid from info_address_port_pair
            // Aggregate data by PID
            let mut pid_stats: HashMap<u32, (u128, u128, u128, u128)> = HashMap::new();
            for (_, val, _) in &search_results {
                if let Some(pids) = &val.pids {
                    for pid in pids {
                        let (total_bytes_in, total_packets_in, total_bytes_out, total_packets_out) = pid_stats
                            .entry(*pid)
                            .or_insert((0, 0, 0, 0));
                        if val.traffic_direction == TrafficDirection::Incoming {
                            *total_bytes_in += val.transmitted_bytes;
                            *total_packets_in += val.transmitted_packets;
                        } else {
                            *total_bytes_out += val.transmitted_bytes;
                            *total_packets_out += val.transmitted_packets;
                        }
                    }
                }
                else {
                    let (total_bytes_in, total_packets_in, total_bytes_out, total_packets_out) = pid_stats
                        .entry(0)
                        .or_insert((0, 0, 0, 0));
                    if val.traffic_direction == TrafficDirection::Incoming {
                        *total_bytes_in += val.transmitted_bytes;
                        *total_packets_in += val.transmitted_packets;
                    } else {
                        *total_bytes_out += val.transmitted_bytes;
                        *total_packets_out += val.transmitted_packets;
                    }
                }
            }

            // apply the filter to the search results after aggregation
            let mut sorted_pids_stats_vec: Vec<(u32, (u128, u128, u128, u128))> = pid_stats.into_iter().collect();
            // make order by most bytes to compare the sum of sum of bytes in and out
            sorted_pids_stats_vec.sort_by(|&(_, a), &(_, b)| match sniffer.report_sort_type {
                ReportSortType::MostBytes => (b.0 + b.2).cmp(&(a.0 + a.2)),
                ReportSortType::MostPackets => (b.1 + b.3).cmp(&(a.1 + a.3)),
                _ => std::cmp::Ordering::Equal,
            });
            let mut scroll_report = Column::new();
            for (pid, (total_bytes_in, total_packets_in, total_bytes_out, total_packets_out)) in &sorted_pids_stats_vec {
                // if pid is 0, mark as unknown
                let pid = if *pid == 0 {
                    "Unknown".to_string()
                } else {
                    pid.to_string()
                };

                let entry_row = Row::new()
                    .align_items(Alignment::Center)
                    .push(
                        Text::new(format!("{:25} {:<10}    ", " ", pid))
                            .style(iced::Color::from_rgb(1.0, 0.5, 0.0))
                            .font(font) // Use a fixed-width font
                            .horizontal_alignment(iced::alignment::Horizontal::Left)
                    )
                    .push(
                        Text::new(format!("{:<15}        {:<15}        {:<15}        {:<15}", get_formatted_bytes_string(*total_bytes_in), total_packets_in, get_formatted_bytes_string(*total_bytes_out), total_packets_out))
                            .style(iced::Color::from_rgb(1.0, 0.5, 0.0))
                            .font(font) // Use a fixed-width font
                            .horizontal_alignment(iced::alignment::Horizontal::Left)
                            .width(Length::Fill),
                    );

                // add entry_row as button to scroll_report
                //convert pid from string to num if possible, if not store value -1 in pid_num
                let pid_num = pid.parse::<u32>().unwrap_or(0);




                scroll_report = scroll_report.push(
                    button(entry_row)
                        .padding(2)
                        .on_press(Message::ShowModal(MyModal::ProcessThrottling(pid_num, ))) //ProcessThrottle(*pid)
                        .style(ButtonStyleTuple(sniffer.style, ButtonType::Neutral).into()),
                );
                //for some wierd the hover effect is not working for the button
                
                
                // scroll_report = scroll_report.push(
                //     entry_row);
            }

            if !sorted_pids_stats_vec.is_empty() {
                col_report = col_report
                    .push(Text::new("  PID       Total Bytes(in)       Total Packets(in)       Total Bytes(out)       Total Packets(out)")
                        .vertical_alignment(Vertical::Center)
                        .horizontal_alignment(Horizontal::Center)
                        .height(Length::FillPortion(2))
                        .font(font) // Use a fixed-width font
                        .width(Length::Fill))
                    .push(Rule::horizontal(5).style(<RuleStyleTuple as Into<iced::theme::Rule>>::into(RuleStyleTuple(
                        sniffer.style,
                        RuleType::Standard,
                    ))))
                    .push(
                        Scrollable::new(scroll_report)
                            .height(Length::FillPortion(15))
                            .width(Length::Fill)
                            .direction(Direction::Both {
                                vertical: ScrollbarType::properties(),
                                horizontal: ScrollbarType::properties(),
                            })
                            .style(
                                <ScrollbarStyleTuple as Into<iced::theme::Scrollable>>::into(
                                    ScrollbarStyleTuple(sniffer.style, ScrollbarType::Standard),
                                ),
                            ),
                    )
                    .push(
                        Rule::horizontal(5).style(<RuleStyleTuple as Into<iced::theme::Rule>>::into(
                            RuleStyleTuple(sniffer.style, RuleType::Standard),
                        )),
                    )
                    .push(get_change_page_row(
                        sniffer.style,
                        sniffer.language,
                        sniffer.page_number,
                        1,  // Update start entry num
                        sorted_pids_stats_vec.len(),  // Update end entry num
                        sorted_pids_stats_vec.len(),
                    ));
            } else {
                col_report = col_report.push(
                    Column::new()
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(20)
                        .align_items(Alignment::Center)
                        .push(vertical_space(Length::FillPortion(1)))
                        .push(Text::new('V'.to_string()).font(ICONS).size(60))
                        .push(vertical_space(Length::Fixed(15.0)))
                        .push(Text::new(no_search_results_translation(sniffer.language)).font(font))
                        .push(vertical_space(Length::FillPortion(2))),
                );
            }
        }
        ReportView::Port => {
        //     // Aggregate data by port
            let mut port_stats: HashMap<u16, (u128, u128, u128, u128)> = HashMap::new();
            for (key, val, _) in &search_results {
                let port = match val.traffic_direction {
                    TrafficDirection::Incoming => key.port2,
                    TrafficDirection::Outgoing => key.port1,
                };
                let (total_bytes_in, total_packets_in, total_bytes_out, total_packets_out) = port_stats
                    .entry(port)
                    .or_insert((0, 0, 0, 0));
                if val.traffic_direction == TrafficDirection::Incoming {
                    *total_bytes_in += val.transmitted_bytes;
                    *total_packets_in += val.transmitted_packets;
                } else {
                    *total_bytes_out += val.transmitted_bytes;
                    *total_packets_out += val.transmitted_packets;
                }
            }

            let mut sorted_ports_stats_vec: Vec<(u16, (u128, u128, u128, u128))> = port_stats.into_iter().collect();




            sorted_ports_stats_vec.sort_by(|&(_, a), &(_, b)| match sniffer.report_sort_type {
                ReportSortType::MostBytes => (b.0 + b.2).cmp(&(a.0 + a.2)),
                ReportSortType::MostPackets => (b.1 + b.3).cmp(&(a.1 + a.3)),
                _ => std::cmp::Ordering::Equal,
            });


            let mut scroll_report = Column::new();
            for (port, (total_bytes_in, total_packets_in, total_bytes_out, total_packets_out)) in &sorted_ports_stats_vec {
                let entry_row = Row::new()
                    .align_items(Alignment::Center)
                    .push(
                        Text::new(format!("{:25} {:<10}    ", " ", port))
                            .style(iced::Color::from_rgb(1.0, 0.5, 0.0))
                            .font(font) // Use a fixed-width font
                            .horizontal_alignment(iced::alignment::Horizontal::Left)
                    )
                    .push(
                        Text::new(format!("{:<15}        {:<15}        {:<15}        {:<15}", get_formatted_bytes_string(*total_bytes_in), total_packets_in, get_formatted_bytes_string(*total_bytes_out), total_packets_out))
                            .style(iced::Color::from_rgb(1.0, 0.5, 0.0))
                            .font(font) // Use a fixed-width font
                            .horizontal_alignment(iced::alignment::Horizontal::Left)
                            .width(Length::Fill),
                    );

                scroll_report = scroll_report.push(entry_row);
            }

            if !sorted_ports_stats_vec.is_empty() {
                col_report = col_report
                    .push(Text::new("  Port       Total Bytes(in)       Total Packets(in)       Total Bytes(out)       Total Packets(out)")
                        .vertical_alignment(Vertical::Center)
                        .horizontal_alignment(Horizontal::Center)
                        .height(Length::FillPortion(2))
                        .font(font) // Use a fixed-width font
                        .width(Length::Fill))
                    .push(Rule::horizontal(5).style(<RuleStyleTuple as Into<iced::theme::Rule>>::into(RuleStyleTuple(
                        sniffer.style,
                        RuleType::Standard,
                    ))))
                    .push(
                        Scrollable::new(scroll_report)
                            .height(Length::FillPortion(15))
                            .width(Length::Fill)
                            .direction(Direction::Both {
                                vertical: ScrollbarType::properties(),
                                horizontal: ScrollbarType::properties(),
                            })
                            .style(
                                <ScrollbarStyleTuple as Into<iced::theme::Scrollable>>::into(
                                    ScrollbarStyleTuple(sniffer.style, ScrollbarType::Standard),
                                ),
                            ),
                    )
                    .push(
                        Rule::horizontal(5).style(<RuleStyleTuple as Into<iced::theme::Rule>>::into(
                            RuleStyleTuple(sniffer.style, RuleType::Standard),
                        )),
                    )
                    .push(get_change_page_row(
                        sniffer.style,
                        sniffer.language,
                        sniffer.page_number,
                        1,  // Update start entry num
                        sorted_ports_stats_vec.len(),  // Update end entry num
                        sorted_ports_stats_vec.len(),
                    ));
            } else {
                col_report = col_report.push(
                    Column::new()
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(20)
                        .align_items(Alignment::Center)
                        .push(vertical_space(Length::FillPortion(1)))
                        .push(Text::new('V'.to_string()).font(ICONS).size(60))
                        .push(vertical_space(Length::Fixed(15.0)))
                        .push(Text::new(no_search_results_translation(sniffer.language)).font(font))
                        .push(vertical_space(Length::FillPortion(2))),
                );
            }
        }
        ReportView::User => {
            // Aggregate data by UID
            let mut uid_stats: HashMap<u32, (u128, u128, u128, u128)> = HashMap::new();
            for (_, val, _) in &search_results {
                if let Some(uid) = val.uid {
                    let (total_bytes_in, total_packets_in, total_bytes_out, total_packets_out) = uid_stats
                        .entry(uid)
                        .or_insert((0, 0, 0, 0));
                    if val.traffic_direction == TrafficDirection::Incoming {
                        *total_bytes_in += val.transmitted_bytes;
                        *total_packets_in += val.transmitted_packets;
                    } else {
                        *total_bytes_out += val.transmitted_bytes;
                        *total_packets_out += val.transmitted_packets;
                    }
                }
                else {
                    let (total_bytes_in, total_packets_in, total_bytes_out, total_packets_out) = uid_stats
                        .entry(MAX) // max value of u32 
                        .or_insert((0, 0, 0, 0));
                    if val.traffic_direction == TrafficDirection::Incoming {
                        *total_bytes_in += val.transmitted_bytes;
                        *total_packets_in += val.transmitted_packets;
                    } else {
                        *total_bytes_out += val.transmitted_bytes;
                        *total_packets_out += val.transmitted_packets;
                    }
                }
            }
            
            let mut sorted_users_stats_vec: Vec<(u32, (u128, u128, u128, u128))> = uid_stats.into_iter().collect();

            sorted_users_stats_vec.sort_by(|&(_, a), &(_, b)| match sniffer.report_sort_type {
                ReportSortType::MostBytes => (b.0 + b.2).cmp(&(a.0 + a.2)),
                ReportSortType::MostPackets => (b.1 + b.3).cmp(&(a.1 + a.3)),
                _ => std::cmp::Ordering::Equal,
            });


            let mut scroll_report = Column::new();
            for (uid, (total_bytes_in, total_packets_in, total_bytes_out, total_packets_out)) in &sorted_users_stats_vec {
                // if uid is MAX, mark as unknown
                let uid = if *uid == MAX {
                    "Unknown".to_string()
                } else {
                    uid.to_string()
                };

                let entry_row = Row::new()
                    .align_items(Alignment::Center)
                    .push(
                        Text::new(format!("{:25} {:<10}    ", " ", uid))
                            .style(iced::Color::from_rgb(1.0, 0.5, 0.0))
                            .font(font) // Use a fixed-width font
                            .horizontal_alignment(iced::alignment::Horizontal::Left)
                    )
                    .push(
                        Text::new(format!("{:<15}        {:<15}        {:<15}        {:<15}", get_formatted_bytes_string(*total_bytes_in), total_packets_in, get_formatted_bytes_string(*total_bytes_out), total_packets_out))
                            .style(iced::Color::from_rgb(1.0, 0.5, 0.0))
                            .font(font) // Use a fixed-width font
                            .horizontal_alignment(iced::alignment::Horizontal::Left)
                            .width(Length::Fill),
                    );

                scroll_report = scroll_report.push(entry_row);
            }
            if !sorted_users_stats_vec.is_empty() {
                col_report = col_report
                    .push(Text::new("  UID       Total Bytes(in)       Total Packets(in)       Total Bytes(out)       Total Packets(out)")
                        .vertical_alignment(Vertical::Center)
                        .horizontal_alignment(Horizontal::Center)
                        .height(Length::FillPortion(2))
                        .font(font) // Use a fixed-width font
                        .width(Length::Fill))
                    .push(Rule::horizontal(5).style(<RuleStyleTuple as Into<iced::theme::Rule>>::into(RuleStyleTuple(
                        sniffer.style,
                        RuleType::Standard,
                    ))))
                    .push(
                        Scrollable::new(scroll_report)
                            .height(Length::FillPortion(15))
                            .width(Length::Fill)
                            .direction(Direction::Both {
                                vertical: ScrollbarType::properties(),
                                horizontal: ScrollbarType::properties(),
                            })
                            .style(
                                <ScrollbarStyleTuple as Into<iced::theme::Scrollable>>::into(
                                    ScrollbarStyleTuple(sniffer.style, ScrollbarType::Standard),
                                ),
                            ),
                    )
                    .push(
                        Rule::horizontal(5).style(<RuleStyleTuple as Into<iced::theme::Rule>>::into(
                            RuleStyleTuple(sniffer.style, RuleType::Standard),
                        )),
                    )
                    .push(get_change_page_row(
                        sniffer.style,
                        sniffer.language,
                        sniffer.page_number,
                        1,  // Update start entry num
                        sorted_users_stats_vec.len(),  // Update end entry num
                        sorted_users_stats_vec.len(),
                    ));
            } else {
                col_report = col_report.push(
                    Column::new()
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(20)
                        .align_items(Alignment::Center)
                        .push(vertical_space(Length::FillPortion(1)))
                        .push(Text::new('V'.to_string()).font(ICONS).size(60))
                        .push(vertical_space(Length::Fixed(15.0)))
                        .push(Text::new(no_search_results_translation(sniffer.language)).font(font))
                        .push(vertical_space(Length::FillPortion(2))),
                );
        }
        }
    }

    Row::new()
        .spacing(15)
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .push(horizontal_space(Length::FillPortion(1)))
        .push(
            Container::new(col_report)
                .padding([10, 7, 7, 7])
                .width(Length::Fixed(1242.0))
                .style(<ContainerStyleTuple as Into<iced::theme::Container>>::into(
                    ContainerStyleTuple(sniffer.style, ContainerType::BorderedRound),
                )),
        )
        .push(
            Container::new(get_button_open_report(
                sniffer.style,
                sniffer.language,
                font,
            ))
            .width(Length::FillPortion(1)),
        )
}


// Function to handle the button message




// fn lazy_report(sniffer: &Sniffer) -> Row<'static, Message> {
//     let font = get_font(sniffer.style);

//     let (search_results, results_number) = get_searched_entries(sniffer);

//     let mut col_report = Column::new()
//         .height(Length::Fill)
//         .width(Length::Fill)
//         .align_items(Alignment::Center);

//     let mut scroll_report = Column::new();
//     let start_entry_num = (sniffer.page_number - 1) * 20 + 1;
//     let end_entry_num = start_entry_num + search_results.len() - 1;
//     for (key, val, flag) in search_results {
//         let sockets_info = get_sockets_info(
//             AddressFamilyFlags::IPV6 | AddressFamilyFlags::IPV4,
//             match key.trans_protocol {
//                 TransProtocol::TCP => ProtocolFlags::TCP,
//                 TransProtocol::UDP => ProtocolFlags::UDP,
//                 TransProtocol::Other => ProtocolFlags::TCP | ProtocolFlags::UDP,
//             },
//         )
//         .unwrap_or_default();
//         let socket_info = sockets_info.iter().find(|s| {
//             let socket_port = match &s.protocol_socket_info {
//                 ProtocolSocketInfo::Tcp(tcp_si) => tcp_si.local_port,
//                 ProtocolSocketInfo::Udp(udp_si) => udp_si.local_port,
//             };
//             socket_port == key.port1
//         });
//         let (uid, pids) = match socket_info {
//             Some(s) => (Some(s.uid), Some(s.associated_pids.clone())),
//             None => (None, None),
//         };
//         dbg!(uid, pids, key.port1);
//         let entry_color = get_connection_color(val.traffic_direction, sniffer.style);
//         let entry_row = Row::new()
//             .align_items(Alignment::Center)
//             .push(
//                 Text::new(format!(
//                     "  {}{}                    ",
//                     key.print_gui(),
//                     val.print_gui()
//                     // TODO: add printing for UIDs and PIDs
//                 ))
//                 .style(iced::theme::Text::Color(entry_color))
//                 .font(font),
//             )
//             .push(flag)
//             .push(Text::new("  "));

//         scroll_report = scroll_report.push(
//             button(entry_row)
//                 .padding(2)
//                 .on_press(Message::ShowModal(MyModal::ConnectionDetails(val.index)))
//                 .style(ButtonStyleTuple(sniffer.style, ButtonType::Neutral).into()),
//         );
//     }
//     if results_number > 0 {
//         col_report = col_report
//             .push(Text::new("      Src IP address       Src port      Dst IP address       Dst port  Layer4   Layer7     Packets     Bytes    pid     uid    Country").vertical_alignment(Vertical::Center).height(Length::FillPortion(2)).font(font))
//             .push(Rule::horizontal(5).style(<RuleStyleTuple as Into<iced::theme::Rule>>::into(RuleStyleTuple(
//                 sniffer.style,
//                 RuleType::Standard,
//             ))))
//             .push(
//                 Scrollable::new(scroll_report)
//                     .height(Length::FillPortion(15))
//                     .width(Length::Fill)
//                     .direction(Direction::Both {
//                         vertical: ScrollbarType::properties(),
//                         horizontal: ScrollbarType::properties(),
//                     })
//                     .style(
//                         <ScrollbarStyleTuple as Into<iced::theme::Scrollable>>::into(
//                             ScrollbarStyleTuple(sniffer.style, ScrollbarType::Standard),
//                         ),
//                     ),
//             )
//             .push(
//                 Rule::horizontal(5).style(<RuleStyleTuple as Into<iced::theme::Rule>>::into(
//                     RuleStyleTuple(sniffer.style, RuleType::Standard),
//                 )),
//             )
//             .push(get_change_page_row(
//                 sniffer.style,
//                 sniffer.language,
//                 sniffer.page_number,
//                 start_entry_num,
//                 end_entry_num,
//                 results_number,
//             ));
//     } else {
//         col_report = col_report.push(
//             Column::new()
//                 .width(Length::Fill)
//                 .height(Length::Fill)
//                 .padding(20)
//                 .align_items(Alignment::Center)
//                 .push(vertical_space(Length::FillPortion(1)))
//                 .push(Text::new('V'.to_string()).font(ICONS).size(60))
//                 .push(vertical_space(Length::Fixed(15.0)))
//                 .push(Text::new(no_search_results_translation(sniffer.language)).font(font))
//                 .push(vertical_space(Length::FillPortion(2))),
//         );
//     }

//     Row::new()
//         .spacing(15)
//         .align_items(Alignment::Center)
//         .width(Length::Fill)
//         .push(horizontal_space(Length::FillPortion(1)))
//         .push(
//             Container::new(col_report)
//                 .padding([10, 7, 7, 7])
//                 .width(Length::Fixed(1242.0))
//                 .style(<ContainerStyleTuple as Into<iced::theme::Container>>::into(
//                     ContainerStyleTuple(sniffer.style, ContainerType::BorderedRound),
//                 )),
//         )
//         .push(
//             Container::new(get_button_open_report(
//                 sniffer.style,
//                 sniffer.language,
//                 font,
//             ))
//             .width(Length::FillPortion(1)),
//         )
// }

fn filters_col(
    search_params: &SearchParameters,
    style: StyleType,
    language: Language,
) -> Column<'static, Message> {
    let font = get_font(style);
    let search_params2 = search_params.clone();

    let mut title_row = Row::new().spacing(10).align_items(Alignment::Center).push(
        Text::new(search_filters_translation(language))
            .font(font)
            .style(TextStyleTuple(style, TextType::Title))
            .size(FONT_SIZE_TITLE),
    );
    if search_params.is_some_filter_active() {
        title_row = title_row.push(button_clear_filter(
            SearchParameters::default(),
            style,
            font,
        ));
    }

    Column::new()
        .spacing(3)
        .push(title_row)
        .push(vertical_space(Length::Fixed(10.0)))
        .push(
            Container::new(
                Checkbox::new(
                    only_show_favorites_translation(language),
                    search_params.only_favorites,
                    move |toggled| {
                        Message::Search(SearchParameters {
                            only_favorites: toggled,
                            ..search_params2.clone()
                        })
                    },
                )
                .spacing(5)
                .size(18)
                .font(font)
                .style(<CheckboxStyleTuple as Into<iced::theme::Checkbox>>::into(
                    CheckboxStyleTuple(style, CheckboxType::Standard),
                )),
            )
            .padding([5, 8])
            .style(<ContainerStyleTuple as Into<iced::theme::Container>>::into(
                ContainerStyleTuple(
                    style,
                    if search_params.only_favorites {
                        ContainerType::Badge
                    } else {
                        ContainerType::Neutral
                    },
                ),
            )),
        )
        .push(
            Row::new()
                .align_items(Alignment::Center)
                .spacing(10)
                .push(filter_input(
                    FilterInputType::App,
                    &search_params.app,
                    application_protocol_translation(language),
                    60.0,
                    search_params.clone(),
                    font,
                    style,
                ))
                .push(filter_input(
                    FilterInputType::Country,
                    &search_params.country,
                    country_translation(language),
                    30.0,
                    search_params.clone(),
                    font,
                    style,
                )),
        )
        .push(
            Row::new()
                .align_items(Alignment::Center)
                .spacing(10)
                .push(filter_input(
                    FilterInputType::Domain,
                    &search_params.domain,
                    domain_name_translation(language),
                    120.0,
                    search_params.clone(),
                    font,
                    style,
                ))
                .push(filter_input(
                    FilterInputType::AS,
                    &search_params.as_name.clone(),
                    administrative_entity_translation(language),
                    120.0,
                    search_params.clone(),
                    font,
                    style,
                )),
        )
}

fn filter_input(
    filter_input_type: FilterInputType,
    filter_value: &str,
    caption: &str,
    width: f32,
    search_params: SearchParameters,
    font: Font,
    style: StyleType,
) -> Container<'static, Message> {
    let is_filter_active = !filter_value.is_empty();

    let button_clear = button_clear_filter(
        match filter_input_type {
            FilterInputType::App => SearchParameters {
                app: String::new(),
                ..search_params.clone()
            },
            FilterInputType::Domain => SearchParameters {
                domain: String::new(),
                ..search_params.clone()
            },
            FilterInputType::Country => SearchParameters {
                country: String::new(),
                ..search_params.clone()
            },
            FilterInputType::AS => SearchParameters {
                as_name: String::new(),
                ..search_params.clone()
            },
        },
        style,
        font,
    );

    let input = TextInput::new("-", filter_value)
        .on_input(move |new_value| {
            Message::Search(match filter_input_type {
                FilterInputType::App => SearchParameters {
                    app: new_value.trim().to_string(),
                    ..search_params.clone()
                },
                FilterInputType::Domain => SearchParameters {
                    domain: new_value.trim().to_string(),
                    ..search_params.clone()
                },
                FilterInputType::Country => SearchParameters {
                    country: new_value.trim().to_string(),
                    ..search_params.clone()
                },
                FilterInputType::AS => SearchParameters {
                    as_name: new_value.trim().to_string(),
                    ..search_params.clone()
                },
            })
        })
        .padding([0, 5])
        .font(font)
        .width(Length::Fixed(width))
        .style(<TextInputStyleTuple as Into<iced::theme::TextInput>>::into(
            TextInputStyleTuple(
                style,
                if is_filter_active {
                    TextInputType::Badge
                } else {
                    TextInputType::Standard
                },
            ),
        ));

    let mut content = Row::new()
        .spacing(5)
        .push(Text::new(format!("{caption}:")).font(font))
        .push(input);

    if is_filter_active {
        content = content.push(button_clear);
    }

    Container::new(content)
        .padding(5)
        .style(<ContainerStyleTuple as Into<iced::theme::Container>>::into(
            ContainerStyleTuple(
                style,
                if is_filter_active {
                    ContainerType::Badge
                } else {
                    ContainerType::Neutral
                },
            ),
        ))
}

fn get_button_change_page(style: StyleType, increment: bool) -> Button<'static, Message> {
    button(
        Text::new(if increment { "j" } else { "i" })
            .size(8.0)
            .font(ICONS)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center),
    )
    .padding(2)
    .height(Length::Fixed(20.0))
    .width(Length::Fixed(25.0))
    .style(ButtonStyleTuple(style, ButtonType::Standard).into())
    .on_press(Message::UpdatePageNumber(increment))
}

fn get_change_page_row(
    style: StyleType,
    language: Language,
    page_number: usize,
    start_entry_num: usize,
    end_entry_num: usize,
    results_number: usize,
) -> Row<'static, Message> {
    Row::new()
        .height(Length::FillPortion(2))
        .align_items(Alignment::Center)
        .spacing(10)
        .padding(0)
        .push(if page_number > 1 {
            Container::new(get_button_change_page(style, false).width(25.0))
        } else {
            Container::new(horizontal_space(25.0))
        })
        .push(
            Text::new(showing_results_translation(
                language,
                start_entry_num,
                end_entry_num,
                results_number,
            ))
            .font(get_font(style)),
        )
        .push(if page_number < (results_number + 20 - 1) / 20 {
            Container::new(get_button_change_page(style, true).width(25.0))
        } else {
            Container::new(horizontal_space(25.0))
        })
}

fn get_button_open_report(
    style: StyleType,
    language: Language,
    font: Font,
) -> Tooltip<'static, Message> {
    let content = button(
        Text::new('8'.to_string())
            .font(ICONS)
            .size(21)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center),
    )
    .padding(10)
    .height(Length::Fixed(50.0))
    .width(Length::Fixed(75.0))
    .style(ButtonStyleTuple(style, ButtonType::Standard).into())
    .on_press(Message::OpenReport);

    Tooltip::new(content, get_open_report_tooltip(language), Position::Top)
        .gap(5)
        .font(font)
        .style(<ContainerStyleTuple as Into<iced::theme::Container>>::into(
            ContainerStyleTuple(style, ContainerType::Tooltip),
        ))
}

fn button_clear_filter(
    new_search_parameters: SearchParameters,
    style: StyleType,
    font: Font,
) -> Button<'static, Message> {
    button(
        Text::new("×")
            .font(font)
            .vertical_alignment(Vertical::Center)
            .horizontal_alignment(Horizontal::Center)
            .size(15),
    )
    .padding(2)
    .height(Length::Fixed(20.0))
    .width(Length::Fixed(20.0))
    .style(ButtonStyleTuple(style, ButtonType::Standard).into())
    .on_press(Message::Search(new_search_parameters))
}

// make a test to check in the process view if the unknown pid really aggregates all None pids
#[cfg(test)]
// start writing the test directly, do not "use"
mod test{
    use crate::Sniffer;
    use crate::gui::pages::inspect_page::lazy_report;

   


}

