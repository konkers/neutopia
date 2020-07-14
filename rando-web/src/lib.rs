#![recursion_limit = "256"]

use wasm_bindgen::prelude::*;
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::{html, prelude::*, ChangeData, Component, ComponentLink, Html, ShouldRender};

use neutopia::verify;

struct Model {
    link: ComponentLink<Self>,

    reader: ReaderService,
    tasks: Vec<ReaderTask>,

    verified_str: String,
}

enum Msg {
    File(Option<File>),
    Loaded(FileData),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            reader: ReaderService::new(),
            tasks: vec![],
            verified_str: "".into(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::File(file) => {
                log::info!("test2");
                if let Some(file) = file {
                    let callback = self.link.callback(Msg::Loaded);
                    let task = self.reader.read_file(file, callback).unwrap();
                    // How does this get cleaned up?
                    self.tasks.push(task);
                }
            }
            Msg::Loaded(file) => {
                self.verified_str = match verify(&file.content) {
                    Ok(info) => format!("{:?}", &info),
                    Err(e) => format!("invalid rom: {}", e),
                }
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <input type="file" multiple=false onchange=self.link.callback(move |value| {
                    let mut result = None;
                    if let ChangeData::Files(files) = value {
                        let file = js_sys::try_iter(&files)
                            .unwrap()
                            .unwrap()
                            .into_iter()
                            .map(|v| File::from(v.unwrap()))
                            .next()
                            .unwrap();
                        result = Some(file);
                    }
                    Msg::File(result)
                })/>
                <p>{ &self.verified_str }</p>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<Model>::new().mount_to_body();
}
