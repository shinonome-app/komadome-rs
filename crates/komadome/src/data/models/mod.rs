mod card;
mod kana_index;
mod list_inp;
mod news;
mod pagination;
mod person;
mod person_all;
mod person_index;
mod top;
mod whatsnew;
mod wip;
mod work_index;

pub use card::*;
pub use list_inp::*;
pub use news::*;
pub use pagination::*;
pub use person::*;
pub use person_all::*;
pub use person_index::*;
pub use top::*;
pub use whatsnew::*;
pub use wip::*;
pub use work_index::*;

#[cfg(test)]
mod tests;
