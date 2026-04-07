use notify_rust::Notification;

pub struct Notif;

impl Notif{
   pub fn send(head:&'static str,body:String)-> std::io::Result<()>{
        let _ = Notification::new()
            .summary(head)
            .body(&body)
            .appname("dex")
            .show();
        Ok(())
    }
}
