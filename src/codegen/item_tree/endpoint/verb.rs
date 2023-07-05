/// A HTTP Verb
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::Display, strum::EnumString)]
#[strum(serialize_all = "UPPERCASE", ascii_case_insensitive)]
pub enum Verb {
    Get,
    Put,
    Post,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}

impl Verb {
    pub(crate) fn request_body_is_legal(self) -> bool {
        match self {
            Verb::Get | Verb::Head | Verb::Delete | Verb::Trace => false,
            Verb::Put | Verb::Post | Verb::Options | Verb::Patch => true,
        }
    }
}
