use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;

use nix::libc;
use nix::poll::{poll, PollFd, PollFlags};

const HIDIOCSFEATURE_BASE: u32 = 0xC0004806;

fn send_feature_report(device: &File, report: &mut [u8]) -> io::Result<()> {
    let length = u32::try_from(report.len())
        .map_err(|_| io::Error::new(
            io::ErrorKind::InvalidInput,
            "Report too large",
        ))?;

    if length > 0x3FFF {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "HID report exceeds ioctl size field",
        ));
    }

    let request = HIDIOCSFEATURE_BASE | (length << 16);

    let result = unsafe {
        libc::ioctl(
            device.as_raw_fd(),
            request as libc::c_ulong,
            report.as_mut_ptr(),
        )
    };

    if result < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

pub fn configure(device: &File) -> io::Result<()> {
    // 1. Enable UART
    let mut enable = [0x41, 0x01];

    send_feature_report(device, &mut enable)?;
    println!("UART enabled");

    // 2. Configure UART: 9600 baud, 8N1
    let mut config = [
        0x50,
        0x00, 0x00, 0x25, 0x80,
        0x00,
        0x00,
        0x03,
        0x00,
        0x00,
    ];

    send_feature_report(device, &mut config)?;
    println!("UART configured: 9600 baud, 8N1");

    // 3. Purge UART buffers
    let mut purge = [0x43, 0x02];

    send_feature_report(device, &mut purge)?;
    println!("UART buffers purged");

    Ok(())
}

pub fn write_uart(device: &mut File, data: &[u8]) -> io::Result<()> {
    if data.is_empty() || data.len() > 63 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "UART payload must contain 1 to 63 bytes",
        ));
    }

    // One byte for the payload length, followed by the UART data
    let mut report = Vec::with_capacity(data.len() + 1);

    report.push(data.len() as u8);
    report.extend_from_slice(data);

    println!("Sending HID report: {report:02X?}");

    device.write_all(&report)?;

    Ok(())
}

pub fn read_uart(device: &mut File) -> io::Result<Option<Vec<u8>>> {
    let mut fds = [PollFd::new(
        device.as_fd(),
        PollFlags::POLLIN,
    )];

    // Wait for at most 1000 milliseconds.
    let result = poll(&mut fds, 1000u16)
        .map_err(io::Error::other)?;

    if result == 0 {
        return Ok(None);
    }

    let events = fds[0]
        .revents()
        .unwrap_or(PollFlags::empty());

    if events.intersects(
        PollFlags::POLLERR
            | PollFlags::POLLHUP
            | PollFlags::POLLNVAL,
    ) {
        return Err(io::Error::new(
            io::ErrorKind::NotConnected,
            "CP2210 disconnected",
        ));
    }

    if !events.contains(PollFlags::POLLIN) {
        return Ok(None);
    }

    let mut report = [0u8; 64];
    let count = device.read(&mut report)?;

    if count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "CP2110 returned an empty HID report",
        ));
    }

    // The first byte specifies the number of UART bytes.
    let payload_len = report[0] as usize;

    if payload_len > report.len() - 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Invalid CP2110 HID report: {payload_len} UART bytes, \
                but only {} bytes available",
                count - 1
            ),
        ));
    }

    Ok(Some(report[1..1 + payload_len].to_vec()))
}


// use nix::ioctl_write_ptr_bad;
//
// // Linux HIDIOCSFEATURE request for a two-byte report.
// //
// // HIDIOCSFEATURE(len) is constructed using Linux's
// // _IOC encoding: WRITE | READ, type 'H', command 0x06
// //
// // For a fixed report length of 2 bytes:
// const HIDIOCSFEATURE_2: u32 = 0xC0024806; // hardcoding Linux specific for now
// // 0xC0024806
// // -       __Command number: 0x06
// // -     __Device class: 'H'
// // -   __Data size: 2 bytes
// // - __Direction: read/write
//
// ioctl_write_ptr_bad!(
//     send_feature_2,
//     HIDIOCSFEATURE_2,
//     u8
// );
//
// pub fn enable_uart(device: &File) -> io::Result<()> {
//     let mut report = [0x41u8, 0x01];
//
//     unsafe {
//         send_feature_2(
//             device.as_raw_fd(),
//             report.as_mut_ptr(),
//         )
//         .map_err(io::Error::other)?;
//     }
//
//     Ok(())
// }
