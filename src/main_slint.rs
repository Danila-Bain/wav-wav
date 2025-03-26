use std::error::Error;

slint::slint! {
    import { Button, VerticalBox, TextEdit, SpinBox, CheckBox, Slider, HorizontalBox } from "std-widgets.slint";

    component Player inherits Rectangle {
        in property <string> filename;

        VerticalBox {

            HorizontalBox {

                max-height: 20px;
                Button {
                    max-width: 20px;
                    text: "▶";
                }

                Text {
                    text: filename;
                }

                Button {
                    text: "Выбрать файл";
                }
            }

            Slider {
                max-width: parent.width;
                height: 20px;
                        value: 42;
            }
        }

    }

    export component AppWindow inherits Window {

        // min-width: 100px;
        // min-height: 100px;
        max-width: 1000px;
        max-height: 1000px;
        in-out property <int> counter: 42;
        callback request-increase-value();

        VerticalBox {

            Player {
                filename: "filename.wav";
            }

            HorizontalBox {
                max-height: 20px;
                CheckBox {
                    text: "Скрыть повторения";
                }

                SpinBox {
                    minimum: 1;
                    maximum: 8;
                    value: 1;
                }

                Button {
                    text: "Расшифровать";
                          clicked => {
                              debug("Расшифровать");
                          }
                }
            }

            TextEdit {
                text: "Result is here";
            }


            HorizontalBox {
                max-height: 20px;
                CheckBox {
                    text: "Повторять";
                }

                SpinBox {
                    minimum: 1;
                    maximum: 8;
                    value: 1;
                }

                Button {
                    text: "Зашифровать";
                }
            }

            Player {
                filename: "filename.wav";
            }

        }
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let logical_size = slint::LogicalSize::new(800.0, 800.0);
    let physical_size = logical_size.to_physical(ui.window().scale_factor());
    ui.window().set_size(physical_size); // don't wait for "Set Size" to be clicked; set the size now!

    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    ui.run()?;

    Ok(())
}
