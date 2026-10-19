mod acquisition;
mod cp2110;
mod protocol;
mod gui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([700.0, 450.0])
            .with_title("UT61E+ Monitor"),

        ..Default::default()
    };

    eframe::run_native(
        "UT61E+ Monitor",
        options,
        Box::new(|_cc| Ok(Box::new(gui::DeviceApp::default()))),
    )
}


// mod cp2110;
// mod protocol;

// use std::fs::OpenOptions;
// use std::io;
// use std::time::{Duration, Instant};




// use protocol::PacketParser;
//
// const GET_NAME: [u8; 6] = [
//     0xAB, 0xCD, 0x03, 0x5F, 0x01, 0xDA,
// ];

// fn main() -> io::Result<()> {
//     let path = "/dev/hidraw4";
//
//     println!("Opening '{path}'...");
//
//     let mut device = OpenOptions::new()
//         .read(true)
//         .write(true)
//         .open(path)?;
//
//     println!("Device '{path}' opened.");
//
//     println!("Enabling CP2110 UART...");
//     cp2110::configure(&device)?;
//     println!("CP2110 initialization complete.");
//
//     // Send the GET_NAME command to the multimeter.
//     cp2110::write_uart(&mut device, &GET_NAME)?;
//
//     let mut parser = PacketParser::new();
//     let mut responses = 0;
//
//     let deadline = Instant::now() + Duration::from_secs(5);
//
//     while Instant::now() < deadline && responses < 2 {
//         match cp2110::read_uart(&mut device)? {
//             Some(report) => {
//                 if report.is_empty() {
//                     continue;
//                 }
//
//                 // First byte gives the number of UART bytes.
//                 let length = report[0] as usize;
//
//                 if length > report.len() - 1 {
//                     eprintln!("Malformed HID report: {report:02X?}");
//                     continue;
//                 }
//
//                 // Feed each UART byte to our protocol parser.
//                 for &byte in &report[1..1 + length] {
//                     if let Some(payload) = parser.push(byte) {
//                         responses += 1;
//
//                         println!("Complete response #{responses}: {payload:02X?}");
//
//                         if let Ok(text) = std::str::from_utf8(&payload) {
//                             println!("ASCII: {text}");
//                         }
//                     }
//                 }
//             }
//
//             None => {
//                 println!("No more data received.");
//                 break;
//             }
//         }
//     }
//
//     if responses < 2 {
//         println!("Received {responses}/2 complete responses before timeout ({deadline:?}).")
//     }
//
//     Ok(())
//
//     // for attempt in 1..=15 {
//     //     println!("Waiting for response ({attempt}/15)...");
//     //
//     //     match cp2110::read_uart(&mut device)? {
//     //         Some(report) => {
//     //             println!("Received {} bytes:", report.len());
//     //             println!("{report:02X?}");
//     //         }
//     //
//     //         None => {
//     //             println!("Timeout: no response.");
//     //             break;
//     //         }
//     //     }
//     // }
//     //
//     // println!("Finished.");
//     //
//     // Ok(())
//
//     // println!("Turn off the meter, then press Enter.");
//     //
//     // loop {
//     //     let mut fds = [PollFd::new(
//     //         device.as_fd(),
//     //         PollFlags::POLLIN,
//     //     )];
//     //
//     //     // Wait up to 500 milliseconds.
//     //     match poll(&mut fds, 500u16) {
//     //         Ok(0) => {
//     //             println!("Timeout: no data.");
//     //         }
//     //
//     //         Ok(_) => {
//     //             let events = fds[0].revents().unwrap_or(PollFlags::empty());
//     //             println!("Events: {events:?}");
//     //
//     //             if events.intersects(
//     //                 PollFlags::POLLERR
//     //                     | PollFlags::POLLHUP
//     //                     | PollFlags::POLLNVAL,
//     //             ) {
//     //                 println!("Device disconnected or unavailable!");
//     //                 break;
//     //             }
//     //
//     //             if events.contains(PollFlags::POLLIN) {
//     //                 println!("Data available!");
//     //             }
//     //         }
//     //
//     //         Err(e) => {
//     //             println!("Polling failed: {e}");
//     //             break;
//     //         }
//     //     }
//     // }
//
//     /*let mut input = String::new();
//     io::stdin().read_line(&mut input)?;*/
//
//     // drop(device);
//     //
//     // println!("Device handle of '{path}' closed.");
// }
