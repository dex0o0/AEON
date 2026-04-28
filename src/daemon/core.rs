// #[allow(unused_imports)]
// use std::io::{Read, Write};
// #[allow(unused_imports)]
// use std::net::TcpStream;
//
//
// pub fn test() -> Result<(), Box<dyn std::error::Error>> {
//     let mut stream = TcpStream::connect("127.0.0.1:3001")?;  // پورت رو عوض کن
//     let request = "GET /status HTTP/1.1\r\nHost: localhost\r\n\r\n";
//     stream.write_all(request.as_bytes())?;
//
//     let mut response = String::new();
//     stream.read_to_string(&mut response)?;
//
//     println!("Response:\n{:?}", response);  // با println چاپ کن
//     // اگه می‌خوای با dbg ببینی:
//     // dbg!(&response);
//
//     Ok(())
// }
