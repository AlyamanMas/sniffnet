#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportView {
    Detailed,
    Summarized,
}

// impl ReportView{
//     fn update(&mut self, message: Message) {
//         match message {
//             Message::ToggleReportView => {
//                 *self = match self {
//                     ReportView::Detailed => ReportView::Summarized,
//                     ReportView::Summarized => ReportView::Detailed,
//                 };
//             }
//             _ => {}
//         }
//     }
// }