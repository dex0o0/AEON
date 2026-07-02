#[macro_use]
pub mod macros;

pub mod daemon {
    pub mod core;
    pub mod log;
    pub mod notif;
}
pub mod modules {
    pub mod backup;
    pub mod monitoring;
}
pub mod port {
    pub mod scan;
}

pub mod socket {
    pub mod lib;
}
