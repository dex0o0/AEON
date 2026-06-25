#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        let _ = $crate::daemon::log::Log::save_log("ERROR", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_sys {
    ($($arg:tt)*) => {
        let _ = $crate::daemon::log::Log::save_log("SYSTEM",format!($($arg)*));
    };
}

#[macro_export]
macro_rules! notif_send {
    ($($arg:tt)*) => {
        if let Err(e) = $crate::daemon::notif::Notif::send("Aeon",format!($($arg)*)){
            $crate::log_error!("Error to send notif:{}",e);

        }
    };
    ($head:expr ,$($arg:tt)*)=>{
        if let Err(e) = $crate::daemon::notif::Notif::send($head,$($arg)*){
            log_error!("Error to send notif:{}",e);
        }
    };
}

#[macro_export]
macro_rules! notif_log_sys {
    ($($arg:tt)*) => {
        log_sys!("{}",$($arg)*);
        notif_send!("{}",$($arg)*);
    };
}
