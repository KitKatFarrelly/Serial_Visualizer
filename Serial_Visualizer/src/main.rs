use eframe::egui;

fn main() -> Result<(), eframe::Error>  {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
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
    name: String,
    age: u32,
}

impl Default for MainFrame {
    fn default() -> Self {
        Self {
            //Put defaults for data in the MainFrame Struct
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}


impl eframe::App for MainFrame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
        });
    }
}