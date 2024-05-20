use eframe::egui;
use egui::{RichText, FontId, Color32};
use serialport::{available_ports, SerialPortType, SerialPort};

fn main() -> Result<(), eframe::Error>  
{
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 480.0]),
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
    num_rows: u32,
    selected_com: String,
    serial_port: Option<Box<dyn SerialPort>>,
}

impl Default for MainFrame 
{
    fn default() -> Self 
    {
        Self 
        {
            //Put defaults for data in the MainFrame Struct
            connect_button_color: Color32::RED,
            num_rows: 30,
            selected_com: "No Ports".to_string(),
            serial_port: None,
        }
    }
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
                    let new_connection = serialport::new(&self.selected_com, 115200).open();
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
                else if self.serial_port.is_some()
                {
                    // Disconnect From Serial Port
                    self.serial_port = None;
                }
            }
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
            //Console Logs at bottom
            ui.heading("Output Log:");
            let default_spacing = ui.spacing().item_spacing.y;
            ui.spacing_mut().item_spacing.y = 0.0;
            egui::ScrollArea::vertical().auto_shrink([false; 2]).max_height(400.0).max_width(1130.0).show(ui, |ui|
            {
                for row in 0..self.num_rows 
                {
                    ui.add(egui::Label::new(RichText::new(format!("{:160}",
                        "placeholder text for console log. Replace with actual console log text."))
                        .color(Color32::GREEN)
                        .background_color(Color32::BLACK)
                        .font(FontId::monospace(12.0))).truncate(true));
                }
            });
            
        });
    }
}
