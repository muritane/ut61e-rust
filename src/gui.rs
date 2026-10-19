use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::sync::mpsc::{self, Receiver, Sender};

const CP2110_VENDOR_ID: u32 = 0x10C4;
const CP2110_PRODUCT_ID: u32 = 0xEA80;

use crate::acquisition::{
    self,
    AcquisitionEvent,
    Command,
};

pub struct DeviceApp {
    devices: Vec<String>,
    last_scan: Instant,

    command_tx: Sender<Command>,
    event_rx: Receiver<AcquisitionEvent>,

    selected_device: Option<String>,
    status: String,
    device_name: Option<String>,
}

impl Default for DeviceApp {
    fn default() -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();

        acquisition::spawn_worker(command_rx, event_tx);

        let devices = scan_devices();

        let selected_device = devices.first().cloned();

        Self {
            devices,
            last_scan: Instant::now(),

            command_tx,
            event_rx,

            selected_device,
            status: "Not queried".to_string(),
            device_name: None,
        }
    }
}

fn scan_devices() -> Vec<String> {
    let Ok(entries) = fs::read_dir("/sys/class/hidraw") else {
        return Vec::new();
    };

    let mut devices: Vec<String> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();

            let uevent_path = entry.path().join("device/uevent");

            let info = fs::read_to_string(uevent_path).ok()?;

            let is_cp2110 = info
                .lines()
                .filter_map(|line| line.strip_prefix("HID_ID="))
                .filter_map(parse_hid_id)
                .any(|(vendor, product)| {
                    vendor == CP2110_VENDOR_ID
                        && product == CP2110_PRODUCT_ID
                });

            is_cp2110.then(|| format!("/dev/{name}"))
        })
        .collect();

    devices.sort();
    devices
}

fn parse_hid_id(id: &str) -> Option<(u32, u32)> {
    let mut fields = id.split(':');

    let _bus = fields.next()?;

    let vendor = u32::from_str_radix(fields.next()?, 16).ok()?;

    let product = u32::from_str_radix(fields.next()?, 16).ok()?;

    Some((vendor, product))
}

impl eframe::App for DeviceApp {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                AcquisitionEvent::Connecting => {
                    self.status = "Connecting...".to_string();
                    self.device_name = None;
                }

                AcquisitionEvent::DeviceIdentified(name) => {
                    self.status = "Last query successful".to_string();
                    self.device_name = Some(name);
                }

                AcquisitionEvent::Timeout => {
                    self.status = "No response from multimeter".to_string();
                }

                AcquisitionEvent::Error(error) => {
                    self.status = format!("Error: {error}");
                }
            }
        }

        //Automatically rescan once per second.
        if self.last_scan.elapsed() >= Duration::from_secs(1) {
            self.devices = scan_devices();
            self.last_scan = Instant::now();

            if self
                .selected_device
                .as_ref()
                .is_none_or(|selected| !self.devices.contains(selected))
            {
                self.selected_device = self.devices.first().cloned();
            }
        }

        // Request another GUI update
        ui.ctx().request_repaint_after(Duration::from_millis(100));

        ui.heading("UT61E+ Monitor");

        ui.separator();

        ui.heading("Available USB devices");

        if self.devices.is_empty() {
            ui.label("No CP2110 devices found");
        } else {
            for device in &self.devices {
                ui.horizontal(|ui| {
                    ui.label("CP2110");
                    ui.label(device);
                    ui.label("USB connected");
                });
            }
        }

        ui.separator();

        if ui.button("Refresh devices").clicked() {
            self.devices = scan_devices();
            self.last_scan = Instant::now();

            if self
                .selected_device
                .as_ref()
                .is_none_or(|selected| !self.devices.contains(selected))
            {
                self.selected_device = self.devices.first().cloned();
            }
        }

        ui.separator();

        ui.heading("Multimeter");

        if self.devices.is_empty() {
            ui.label("No CP2110 devices found.");
        } else {
            egui::ComboBox::from_label("Device")
                .selected_text(
                    self.selected_device
                        .as_deref()
                        .unwrap_or("Select device"),
                )
                .show_ui(ui, |ui| {
                    for device in &self.devices {
                        ui.selectable_value(
                            &mut self.selected_device,
                            Some(device.clone()),
                            device,
                        );
                    }
                });

            if ui.button("Query Device").clicked() {
                if let Some(path) = &self.selected_device {
                    if self
                        .command_tx
                        .send(Command::QueryDevice(path.clone()))
                        .is_ok()
                    {
                        self.status = "Query queued...".to_string();
                    } else {
                        self.status = "Acquisition worker stopped".to_string();
                    }
                }
            }
        }

        ui.label(format!("Status: {}", self.status));

        if let Some(name) = &self.device_name {
            ui.label(format!("Last identified device: {name}"));
        }
    }
}
