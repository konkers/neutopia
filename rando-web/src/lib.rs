#![recursion_limit = "512"]

use regex::Regex;
use wasm_bindgen::prelude::*;
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::web_sys::Blob;
use yew::{html, prelude::*, ChangeData, Component, ComponentLink, Html, ShouldRender};

use neutopia::verify;
use rando::{randomize, Config, RandoType};

mod info;

use info::Info;

#[derive(Debug, PartialEq)]
enum WebsiteMode {
    Dev,
    Beta,
    Release,
}

fn parse_semver(semver: &str) -> WebsiteMode {
    let re = Regex::new(r"^(v\d+\.\d+\.\d+)(-.*)?$").unwrap();
    let captures = re.captures(semver);
    match captures {
        Some(c) => {
            if let Some(_) = c.get(2) {
                WebsiteMode::Beta
            } else {
                WebsiteMode::Release
            }
        }
        None => WebsiteMode::Dev,
    }
}

fn get_website_mode() -> WebsiteMode {
    parse_semver(env!("VERGEN_SEMVER_LIGHTWEIGHT"))
}

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

impl Model {
    fn view_coming_soon(&self) -> Html {
        html! {
            <nav class="panel is-primary">
                <p class="panel-heading">
                    {"Comming Soon"}
                </p>
                <div class="panel-block">
                    <p>
                        {"The Neutopia randomizer is under heavy development.  We don't
                          stable release yet.  If you'd like to try the beta head over to "}
                        <a href="https://beta.neutopia.run">{"beta.neutopia.run"}</a>
                        {" to generate a seed."}
                    </p>
                </div>
            </nav>
        }
    }

    fn view_randomizer(&self) -> Html {
        html! {
            <>
                <Info />
                <nav class="panel is-primary">
                    <p class="panel-heading">
                       {"Generate Seed"}
                    </p>
                    <div class="panel-block">
                        {"Options will go here"}
                    </div>
                    <div class="panel-block">
                        <div class="file">
                            <span class="file-cta">
                                <span class="file-icon">
                                    <i class="mdi mdi-folder-open"></i>
                                </span>
                                <span class="file-label">{"Select US Neutopia rom"}</span>
                            </span>
                            <input class="file-input" type="file" multiple=false onchange=self.link.callback(move |value| {
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
                        </div>
                    </div>
                </nav>
            </>
        }
    }
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
                if let Some(file) = file {
                    let callback = self.link.callback(Msg::Loaded);
                    let task = self.reader.read_file(file, callback).unwrap();
                    // How does this get cleaned up?
                    self.tasks.push(task);
                }
            }
            Msg::Loaded(file) => {
                self.verified_str = match verify(&file.content) {
                    Ok(info) => {
                        let config = Config {
                            ty: RandoType::Global,
                            seed: None,
                        };
                        let game = randomize(&config, &file.content).unwrap();

                        saveRom(
                            &game.data,
                            format!("neutopia-randomizer-{}.pce", game.seed).into(),
                        );
                        format!("{:?}", &info)
                    }
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
            <div class="content">
                <div class="logo">
                    <img src="logo.png"/>
                </div>
                { match get_website_mode() {
                    WebsiteMode::Release => self.view_coming_soon(),
                    _ => self.view_randomizer(),
                }}
                <section class="section">
                    <div class="container">
                        <p>{ &self.verified_str }</p>
                    </div>
                    <div class="container">
                        <p>{
                            match get_website_mode(){
                             WebsiteMode::Dev => format!("dev {} {}", env!("VERGEN_SHA"), env!("VERGEN_BUILD_TIMESTAMP")),
                             _ => format!("{} {} {}", env!("VERGEN_SEMVER_LIGHTWEIGHT"), env!("VERGEN_SHA"), env!("VERGEN_BUILD_TIMESTAMP")),
                           }
                        }</p>
                    </div>
                </section>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<Model>::new().mount_to_body();
}

#[wasm_bindgen]
extern "C" {
    fn saveAs(blob: Blob, filename: String);
    fn saveRom(data: &[u8], filename: String);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_website_mode() {
        assert_eq!(parse_semver("v0.1.0-beta1"), WebsiteMode::Beta);
        assert_eq!(parse_semver("v0.1.0"), WebsiteMode::Release);
        assert_eq!(parse_semver("UNKNOWN"), WebsiteMode::Dev);
    }
}
