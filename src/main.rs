use std::time::Duration;

use eframe::egui;
use serde::{Deserialize, Serialize};
use serialport::{self, SerialPort};
use postcard::{to_vec, from_bytes};

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let ports = serialport::available_ports().expect("No ports found!");
    let mut port = serialport::new("/dev/ttyACM1", 115_200)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open port");

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            // egui_extras::install_image_loaders(&cc.egui_ctx);
            let app = MyApp {
                port,
                leds_state: LedsState::default()
            };
            Ok(Box::new(app))
        }),
    )
}

struct MyApp {
    port: Box<dyn SerialPort>,
    leds_state: LedsState
}

struct LedsState {
    red: bool,
    blue: bool,
    green: bool
}

impl Default for LedsState {
    fn default() -> Self {
        LedsState {
            red: false,
            blue: false,
            green: false
        }
    }
}

#[derive(Serialize, Deserialize)]
enum Led {
    Red,
    Blue,
    Green
}

#[derive(Serialize, Deserialize)]
enum Request {
    TurnOn(Led),
    TurnOff(Led)
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");

            if self.leds_state.blue {
                if ui.button("Turn off blue").clicked() {
                    let output = Request::TurnOff(Led::Blue);
                    let buf = to_vec::<Request, 32>(&output).unwrap();
                    println!("{:?}", buf);
                    self.port.write(&buf).expect("Write failed!");
                    self.leds_state.blue = !self.leds_state.blue;
                }
            } else {
                if ui.button("Turn on blue").clicked() {
                    let output = Request::TurnOn(Led::Blue);
                    let buf = to_vec::<Request, 32>(&output).unwrap();
                    println!("{:?}", buf);
                    self.port.write(&buf).expect("Write failed!");
                    self.leds_state.blue = !self.leds_state.blue;
                }
            }

            if self.leds_state.red {
                if ui.button("Turn off red").clicked() {
                    let output = Request::TurnOff(Led::Red);
                    let buf = to_vec::<Request, 32>(&output).unwrap();
                    println!("{:?}", buf);
                    self.port.write(&buf).expect("Write failed!");
                    self.leds_state.red = !self.leds_state.red;
                }
            } else {
                if ui.button("Turn on red").clicked() {
                    let output = Request::TurnOn(Led::Red);
                    let buf = to_vec::<Request, 32>(&output).unwrap();
                    println!("{:?}", buf);
                    self.port.write(&buf).expect("Write failed!");
                    self.leds_state.red = !self.leds_state.red;
                }
            }

            if self.leds_state.green {
                if ui.button("Turn off green").clicked() {
                    let output = Request::TurnOff(Led::Green);
                    let buf = to_vec::<Request, 32>(&output).unwrap();
                    println!("{:?}", buf);
                    self.port.write(&buf).expect("Write failed!");
                    self.leds_state.green = !self.leds_state.green;
                }
            } else {
                if ui.button("Turn on green").clicked() {
                    let output = Request::TurnOn(Led::Green);
                    let buf = to_vec::<Request, 32>(&output).unwrap();
                    println!("{:?}", buf);
                    self.port.write(&buf).expect("Write failed!");
                    self.leds_state.green = !self.leds_state.green;
                }
            }
        });
    }
}
