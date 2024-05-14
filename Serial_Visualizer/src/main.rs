use eframe::egui;
use egui::{RichText, FontId, Color32};

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
                //send some serial data through COM Port. Change Color once connected.
                self.connect_button_color = Color32::GREEN;
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
