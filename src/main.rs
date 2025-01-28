use std::{time::Duration};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};

use eframe::egui::{self, output};
use serde::{Deserialize, Serialize};
use serialport::{self, SerialPort};
use postcard::{to_vec, from_bytes};
use serde_json::{json, Value};


fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    let mut json_array: Vec<Value> = Vec::new();

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            // egui_extras::install_image_loaders(&cc.egui_ctx);
            let app = MyApp {
                port: None,
                json_array,
            };
            Ok(Box::new(app))
        }),
    )
}

struct MyApp {
    port: Option<Box<dyn SerialPort>>,
    json_array: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
enum E {
    SomeError
}

#[derive(Serialize, Deserialize)]
enum Request {
    StartSending,
    GetMessage,
    SendingCompleted
}

#[derive(Serialize, Deserialize)]
enum Response {
    SendingStarted,
    Message(Option<LogMessage>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Gyro {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum LogMessage {
    Accel {
        x: f32,
        y: f32,
        z: f32,
    },
    Gyro {
        x: f32,
        y: f32,
        z: f32,
    },
    Mag {
        x: f32,
        y: f32,
        z: f32,
    },
    Motors {
        m1: f32,
        m2: f32,
        m3: f32,
        m4: f32,
    },
    Batt {
        v: f32,
        a: f32,
    },
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");

            match &mut self.port {
                Some(port) => {
                    if ui.button("get messages").clicked() {
        
                        let mut file_content = String::new();
                        let mut file = OpenOptions::new()
                            .create(true)
                            .read(true)
                            .write(true)
                            .open("log_file.json").unwrap();
        
                        let output = Request::StartSending;
                        let buf = to_vec::<Request, 32>(&output).unwrap();
                        port.write(&buf).expect("Write failed!");
        
                        let mut buf = [0_u8; 32];
                        let n = port.read(&mut buf).unwrap();
                        let response: Response = from_bytes(&buf[..n]).unwrap();
                        match response {
                            Response::SendingStarted => {
                                file.read_to_string(&mut file_content).unwrap();

                                self.json_array = Vec::new();
        
                                let mut json_array = if file_content.is_empty() {
                                    Vec::new()
                                } else {
                                    serde_json::from_str(&file_content).unwrap_or_else(|_| Vec::new())
                                };
        
                                loop {
                                    let output = Request::GetMessage;
                                    let buf = to_vec::<Request, 32>(&output).unwrap();
                                    port.write(&buf).expect("Write failed!");
                
                                    let mut buf = [0_u8; 32];
                                    let n = port.read(&mut buf).unwrap();
                                    let response: Result<Response, postcard::Error> = from_bytes(&buf[..n]);
                
                                    match response {
                                        Ok(r) => {
                                            match r {
                                                Response::Message(log_message) => {
                                                    match log_message {
                                                        Some(m) => {
                                                            let value: Value = serde_json::to_value(&m).unwrap();
                                                            json_array.push(value.clone());
                                                            self.json_array.push(value);
                                                        },
                                                        None => break,
                                                    }
                                                },
                                                _ => ()
                                            }
                                        },
                                        Err(_) => ()
                                    }
                                }
                                
                                let mut file = File::create("log_file.json").unwrap();
                                writeln!(file, "{}", serde_json::to_string_pretty(&self.json_array).unwrap()).unwrap();
                            },
                            _ => ()
                        }
                    }
                },
                None => {
                    if ui.button("Initialize Port").clicked() {
                        // Attempt to initialize the port when the button is clicked
                        match serialport::new("/dev/ttyACM1", 115_200)
                            .timeout(Duration::from_millis(100))
                            .open()
                        {
                            Ok(port) => {
                                self.port = Some(port);
                                ui.label("Port initialized successfully!");
                            }
                            Err(e) => {
                                ui.label(format!("Failed to initialize port: {}", e));
                            }
                        }
                    }
                },
            }

            // Add a scrollable area for the JSON data
            egui::ScrollArea::vertical()
                .min_scrolled_width(300.0)
                .show(ui, |ui| {
                    for value in &self.json_array {
                        display_json_value(ui, value, 0); // Display JSON data with indentation
                    }
                });
        });
    }
}

// Helper function to display JSON values with indentation
fn display_json_value(ui: &mut egui::Ui, value: &serde_json::Value, indent_level: usize) {
    match value {
        serde_json::Value::Object(obj) => {
            ui.vertical(|ui| {
                for (key, val) in obj {
                    ui.horizontal(|ui| {
                        ui.add_space(indent_level as f32 * 10.0); // Add indentation
                        ui.label(format!("{}:", key));
                        display_json_value(ui, val, indent_level + 1); // Increase indentation for nested values
                    });
                }
            });
        }
        serde_json::Value::Array(arr) => {
            ui.vertical(|ui| {
                for (index, val) in arr.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add_space(indent_level as f32 * 10.0); // Add indentation
                        ui.label(format!("[{}]:", index));
                        display_json_value(ui, val, indent_level + 1); // Increase indentation for nested values
                    });
                }
            });
        }
        serde_json::Value::String(s) => {
            ui.horizontal(|ui| {
                ui.add_space(indent_level as f32 * 10.0); // Add indentation
                ui.label(s);
            });
        }
        serde_json::Value::Number(n) => {
            ui.horizontal(|ui| {
                ui.add_space(indent_level as f32 * 10.0); // Add indentation
                ui.label(n.to_string());
            });
        }
        serde_json::Value::Bool(b) => {
            ui.horizontal(|ui| {
                ui.add_space(indent_level as f32 * 10.0); // Add indentation
                ui.label(b.to_string());
            });
        }
        serde_json::Value::Null => {
            ui.horizontal(|ui| {
                ui.add_space(indent_level as f32 * 10.0); // Add indentation
                ui.label("null");
            });
        }
    }
}

