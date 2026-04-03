use notify_rust::Notification;


pub struct Notif{
    head:String,
    body:String,
}

impl Notif{
    pub fn send(self)-> std::io::Result<()>{
        let _ = Notification::new()
            .summary(&self.head)
            .body(&self.body)
            .appname("dex")
            .show();
        Ok(())
    }
}
