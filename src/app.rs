use std::{path::Path, sync::LazyLock};

use anyhow::Result;

use crate::{
    components::{
        Component,
        core::{
            body::Body,
            div::Div,
            router_view::RouterView,
            transition::{Transition, TransitionMode},
        },
        md::{
            icon::Icon,
            layout::Layout,
            list::{Divider, List, ListItem},
            main::Main,
            navigation_drawer::NavigationDrawer,
            snackbar::Snackbar,
            spacer::Spacer,
        },
    },
    router::Router,
    styles::{
        Style,
        measurement::Measurement,
        tokens::{COLOR_BACKGROUND, COLOR_PRIMARY},
    },
    window::Window,
};

#[derive(Eq, PartialEq)]
pub enum Routes {
    Index,
    Calendar,
    Locations,
    People,
    Settings,
    Stats,
    Tagger,
    Tags,
}

pub static ROUTER: LazyLock<Router<Routes>> = LazyLock::new(|| Router::new(Routes::Index));

pub fn create_app() -> Result<()> {
    let window = Window::new(
        "Photo Manager",
        (1200, 900),
        Path::new("icons/32x32.png").to_path_buf(),
    );

    let mut body = Body::new(vec![
        &Layout::new(vec![
            NavigationDrawer::new(vec![
                List::new(vec![
                    ListItem::new()
                        .prepend_icon(Icon::Image)
                        .title("Photos")
                        .to(Routes::Tagger),
                    ListItem::new()
                        .prepend_icon(Icon::MapMarker)
                        .title("Locations")
                        .to(Routes::Locations),
                    ListItem::new()
                        .prepend_icon(Icon::Calendar)
                        .title("Calendar")
                        .to(Routes::Calendar),
                    &Divider::new(),
                    ListItem::new()
                        .prepend_icon(Icon::Tag)
                        .title("Tags")
                        .to(Routes::Tags),
                    ListItem::new()
                        .prepend_icon(Icon::Account)
                        .title("People")
                        .to(Routes::People),
                    ListItem::new()
                        .prepend_icon(Icon::ChartLine)
                        .title("Statistics")
                        .to(Routes::Stats),
                ])
                .style(&Style::new().height(Measurement::Calc(|viewport| {
                    Measurement::Vh(1.0).resolve(viewport) - Measurement::Px(128).resolve(viewport)
                })))
                .color(COLOR_PRIMARY)
                .nav(true),
                &Spacer::new(),
                List::new(vec![
                    ListItem::new()
                        .prepend_icon(Icon::Cog)
                        .title("Settings")
                        .to(Routes::Settings),
                    ListItem::new()
                        .prepend_icon(Icon::ExitToApp)
                        .title("Close Project")
                        .to(Routes::Index),
                ])
                .color(COLOR_PRIMARY)
                .nav(true),
            ])
            .expand_on_hover(true)
            .permanent(true)
            .rail(true)
            .cond(|| ROUTER.route != Routes::Index),
            &Main::new(vec![RouterView::new().component(|component| {
                let mut div = Div::new(vec![component]);
                div.style(&Style::new().height(Measurement::Percent(1.0)));
                let mut t = Transition::new(vec![&div]);
                t.mode(TransitionMode::OutIn);
                Box::new(t) as Box<dyn Component>
            })]),
        ]),
        &Snackbar::new(vec![]),
    ]);
    body.style(
        Style::new()
            .background_color(COLOR_BACKGROUND)
            .width(Measurement::Percent(1.0))
            .height(Measurement::Percent(1.0)),
    );

    window.build(body)?;

    Ok(())
}
