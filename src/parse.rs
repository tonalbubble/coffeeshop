use crate::model::{Coffee, Size};

//simple file to parse strings into enum types
pub fn parse_coffee(s: &str) -> Coffee {
    match s {
        "Columbian"      => Coffee::Columbian,
        "Arabica"        => Coffee::Arabica,
        "Robusta"        => Coffee::Robusta,
        "Excelsa"        => Coffee::Excelsa,
        "BreakfastBlend" => Coffee::BreakfastBlend,
        "MidnightRoast"  => Coffee::MidnightRoast,
        _                => Coffee::Arabica,
    }
}

pub fn parse_size(s: &str) -> Size {
    match s {
        "Small"  => Size::Small,
        "Medium" => Size::Medium,
        "Large"  => Size::Large,
        _        => Size::Medium,
    }
}