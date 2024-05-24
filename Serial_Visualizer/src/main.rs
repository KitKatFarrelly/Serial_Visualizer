use eframe::egui::{self, ecolor::ecolor_assert};
use egui::{RichText, FontId, Color32};
use serialport::{available_ports, SerialPortType, SerialPort};
use std::collections::VecDeque;
use std::num::NonZeroI128;
use std::time::Duration;

fn main() -> Result<(), eframe::Error>  
{
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Confirm exit",
        options,
        Box::new(|_cc| Box::<MainFrame>::default()),
    )
}

struct MainFrame
{
    //Put State here
    connect_button_color: Color32,
    selected_com: String,
    serial_port: Option<Box<dyn SerialPort>>,
    console_log: VecDeque<String>,
    last_incomplete_msg: Option<Vec<u8>>,
    console_log_iter: usize,
    input_text: String,
    currently_reading_raw: bool,
    raw_start_idx: i32,
    current_raw_size: i32,
    //Displayed Data
    tof_frame_matrix: Vec<u32>,
    imu_timestamp: u32,
    accel_matrix: Vec<i32>,
    gyro_matrix: Vec<i32>,
}

trait InternalHandlers
{
    fn handleRawData(&mut self, raw_frame: Vec<u8>);
}

impl Default for MainFrame 
{
    fn default() -> Self 
    {
        Self 
        {
            //Put defaults for data in the MainFrame Struct
            connect_button_color: Color32::RED,
            selected_com: "No Ports".to_string(),
            serial_port: None,
            console_log: VecDeque::from(vec!["".to_string(); 30]),
            last_incomplete_msg: None,
            console_log_iter: 0,
            input_text: "".to_string(),
            currently_reading_raw: false,
            raw_start_idx: 0,
            current_raw_size: 0,
            //Displayed Data
            tof_frame_matrix: vec![0;64],
            imu_timestamp: 0,
            accel_matrix: vec![0;3],
            gyro_matrix: vec![0;3],
        }
    }
}

fn testChecksum(raw_frame: &Vec<u8>) -> bool
{
    let mut checksum: u8 = 0;
    for dat in raw_frame
    {
        checksum = checksum ^ dat;
    }
    return checksum == 0;
}

fn returnUartList() -> Vec<String>
{
    let mut usbComList = Vec::<String>::new();
    match available_ports()
    {
        Ok(ports) => 
        {
            
            for p in ports
            {
                if let SerialPortType::UsbPort(info) = p.port_type
                {
                    //append name to list
                    usbComList.push(p.port_name);
                }
            }
            
        }
        Err(e) => {}
    }
    return usbComList;
}

impl InternalHandlers for MainFrame
{
    fn handleRawData(&mut self, raw_frame: Vec<u8>)
    {
        //send raw data frames to their proper handler.
        match raw_frame[5]
        {
            0 =>
            {
                //empty timestamp from imu
            },
            1 =>
            {
                //only acceleration data
            },
            2 =>
            {
                //only gyro data
            },
            3 =>
            {
                //both acc and gyro data
            },
            4 =>
            {
                //tof data
            },
            _ =>
            {
                //default
                println!("invalid data type");
            }
        }
    }
}

impl eframe::App for MainFrame 
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) 
    {
        egui::CentralPanel::default().show(ctx, |ui|
        {
            if ui.add(egui::Button::new(RichText::new(format!("Connect To Robot")).color(Color32::BLACK).font(FontId::proportional(20.0))).fill(self.connect_button_color)).clicked()
            {
                if self.serial_port.is_none()
                {
                    // Connect To Serial Port
                    let new_connection = serialport::new(&self.selected_com, 115200).timeout(Duration::from_millis(10)).open();
                    match new_connection
                    {
                        Ok(conn) => 
                        {
                            self.serial_port = Some(conn);
                            self.connect_button_color = Color32::GREEN;
                        },
                        Err(e) => {eprintln!("Failed to open \"{}\". Error: {}", &self.selected_com, e)},
                    }
                    
                }
                else 
                {
                    // Disconnect From Serial Port
                    self.serial_port = None;
                }
            }
            //Business logic for Serial First, after running connection logic
            if self.serial_port.is_none() || !returnUartList().contains(&self.selected_com)
            {
                self.connect_button_color = Color32::RED;
                self.serial_port = None;
                egui::ComboBox::from_id_source("my-combobox")
                    .selected_text(format!("{}", self.selected_com))
                    .show_ui(ui, |ui|
                    {
                        let sel_com_borrow = &mut self.selected_com;
                        for p in returnUartList()
                        {
                            let str_copy = p.clone();
                            ui.selectable_value(sel_com_borrow, p, str_copy);
                        }
                    });
            }
            else
            {
                //Read up to 1000 bytes and slice into lines
                let mut serial_buf: Vec<u8> = vec![0; 1000];
                match self.serial_port.as_mut().unwrap().bytes_to_read()
                {
                    Ok(num_bytes) =>
                    {
                        if num_bytes > 0
                        {
                            match self.serial_port.as_mut().unwrap().read(serial_buf.as_mut_slice())
                            {
                                Ok(t) => 
                                {
                                    let mut buf_lower_iter = 0;
                                    for buf_iter in 0..t
                                    {
                                        //check if we're reading a raw line first. raw data needs to be handled differently.
                                        if self.currently_reading_raw
                                        {
                                            if self.current_raw_size == 0 && ((buf_iter as i32 - self.raw_start_idx) == 4)
                                            {
                                                self.current_raw_size = serial_buf[buf_iter] as i32;
                                            }
                                            if (buf_iter as i32 - self.raw_start_idx) >= (self.current_raw_size + 8)
                                            {
                                                let mut raw_vec = Vec::new();
                                                if self.last_incomplete_msg.is_some()
                                                {
                                                    raw_vec.extend(self.last_incomplete_msg.as_ref().unwrap());
                                                    self.last_incomplete_msg = None;
                                                }
                                                if buf_iter - buf_lower_iter > 1
                                                {
                                                    raw_vec.extend_from_slice(&serial_buf[buf_lower_iter..(buf_iter - 1)]);
                                                }
                                                //at this point, we can send the raw data vector to the data handler.
                                                if(testChecksum(&raw_vec))
                                                {
                                                    InternalHandlers::handleRawData(self, raw_vec);
                                                }
                                                self.currently_reading_raw = false;
                                                self.current_raw_size = 0;
                                                buf_lower_iter = buf_iter; //technically ends at first byte of next string
                                            }
                                            else
                                            {
                                                continue
                                            }
                                        }
                                        //check if line feed or carriage return or raw data start and end line there
                                        else if serial_buf[buf_iter] == 0x0A || serial_buf[buf_iter] == 0x0D || serial_buf[buf_iter] == 0xFE
                                        {
                                            if buf_iter - buf_lower_iter > 1
                                            {
                                                //need to check for invalid characters in these eventually
                                                let mut str_vec = Vec::new();
                                                if self.last_incomplete_msg.is_some()
                                                {
                                                    str_vec.extend(self.last_incomplete_msg.as_ref().unwrap());
                                                    self.last_incomplete_msg = None;
                                                }
                                                str_vec.extend_from_slice(&serial_buf[buf_lower_iter..(buf_iter-1)]);
                                                match String::from_utf8(str_vec)
                                                {
                                                    Ok(full_str) =>
                                                    {
                                                        if self.console_log_iter < self.console_log.len()
                                                        {
                                                            
                                                            self.console_log[self.console_log_iter] = full_str;
                                                        }
                                                        else
                                                        {
                                                            self.console_log.push_back(full_str);
                                                        }
                                                    }
                                                    Err(e) =>
                                                    {
                                                        println!("not a valid utf-8 string, dropping.");
                                                    }
                                                }
                                                if(self.console_log_iter < 2000)
                                                {
                                                    self.console_log_iter += 1;
                                                }
                                                else
                                                {
                                                    self.console_log.pop_front();
                                                }
                                            }
                                            buf_lower_iter = buf_iter + 1;
                                            if serial_buf[buf_iter] == 0xFE
                                            {
                                                buf_lower_iter = buf_iter;
                                                self.currently_reading_raw = true;
                                                self.raw_start_idx = buf_iter as i32;
                                            }
                                        }
                                    }
                                    if buf_lower_iter < t
                                    {
                                        self.last_incomplete_msg = Some(serial_buf[buf_lower_iter..t].to_vec());
                                        if self.raw_start_idx > 0
                                        {
                                            self.raw_start_idx = self.raw_start_idx - t as i32;
                                        }
                                    }
                                },
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                    },
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            //Console Logs at bottom
            ui.heading("Output Log:");
            let default_spacing = ui.spacing().item_spacing.y;
            ui.spacing_mut().item_spacing.y = 0.0;
            egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false; 2]).max_height(400.0).max_width(1130.0).show(ui, |ui|
            {
                for row in 0..self.console_log.len() 
                {
                    ui.add(egui::Label::new(RichText::new(format!("{:160}",
                        self.console_log[row]))
                        .color(Color32::GREEN)
                        .background_color(Color32::BLACK)
                        .font(FontId::monospace(12.0))).truncate(true));
                }
            });
            ui.spacing_mut().item_spacing.y = default_spacing;
            ui.add_space(8.0);
            //Text box to send text with
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.input_text).hint_text("send command"));
                if ui.add(egui::Button::new("Send")).clicked()
                {
                    if self.serial_port.is_some()
                    {
                        self.input_text.push_str("\n");
                        match self.serial_port.as_mut().unwrap().write(self.input_text.as_bytes()) {
                            Ok(_) => {
                                println!("{}", &self.input_text);
                                self.input_text = "".to_string();
                            }
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                }
            });
            ui.end_row();
            ctx.request_repaint();
        });
    }
}
