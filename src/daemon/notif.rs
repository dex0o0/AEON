use notify_rust::Notification;

#[allow(dead_code)]
pub struct Notif;

#[allow(dead_code)]
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
