// import language
use crate::Language;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum ReportView {
    Detailed,
    Process,
    User,
    Port,
}

// implement string conversion for ReportView
impl std::fmt::Display for ReportView {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ReportView::Detailed => write!(f, "Detailed"),
            ReportView::Process => write!(f, "Process"),
            ReportView::User => write!(f, "User"),
            ReportView::Port => write!(f, "Port"),
        }
    }
}

// implement ALL and get_radio_label for ReportView
impl ReportView {
    pub(crate) const ALL: [ReportView; 4] = [
        ReportView::Detailed,
        ReportView::Process,
        ReportView::User,
        ReportView::Port,
    ];

    pub fn get_radio_label(&self, language: Language) -> &str {
        match self {
            ReportView::Detailed => "Detailed",
            ReportView::Process => "Process",
            ReportView::User => "User",
            ReportView::Port => "Port",
        }
    }
}