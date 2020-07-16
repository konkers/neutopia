use yew::{html, Component, ComponentLink, Html, ShouldRender};

pub struct Info {
    _link: ComponentLink<Self>,
}

impl Component for Info {
    type Message = ();
    type Properties = ();
    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Info { _link: link }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <nav class="panel is-primary">
                <p class="panel-heading">
                    {"Information"}
                </p>
                <div class="panel-block">
                <p>
                   {"Neutopia randomizer is in in very early development.  Currently the only thing that is randomized is key location.  Care has been taken to avoid un-completable seeds.   If you find a bug, please feel free to file and issue on our "}
                   <a href="https://github.com/konkers/neutopia/issues">{"tracker"}</a>{"."}
                    </p>
                </div>
            </nav>

        }
    }
}
