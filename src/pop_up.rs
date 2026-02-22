use egor::app::{egui::Id, egui::Modal, egui::Ui};

pub struct PopUp {
    pub heading: String,
    pub msg: String,
    pub visible: bool,
}

impl PopUp {
    pub fn ui(&mut self, ui: &mut Ui) {
        if self.visible {
            let modal = Modal::new(Id::new("popup")).show(ui.ctx(), |ui| {
                ui.set_width(200.);
                ui.heading(&self.heading);
                ui.add_space(18.);

                ui.label(&self.msg);
                if ui.button("Ok").clicked() {
                    ui.close();
                }
            });

            if modal.should_close() {
                self.visible = false;
            }
        }
    }
}
