use seed::{prelude::*, *};
use strum::IntoEnumIterator;
use web_sys::{HtmlCanvasElement, HtmlInputElement};

use crate::qoi::QoiChunk;
use crate::static_image::StaticImage;
use crate::util;
use crate::vis::{color_of_chunk, visualize, VisConfig};

#[derive(Debug)]
struct Model {
    img: StaticImage,
    config: VisConfig,
    refs: Refs,
}

#[derive(Debug, Default)]
struct Refs {
    input_file: ElRef<HtmlInputElement>,
    canvas: ElRef<HtmlCanvasElement>,
}

#[derive(Debug)]
enum Msg {
    InputFileChanged,
    UpdateImage(StaticImage),
    ToggleChunkVisibility(QoiChunk),
    MakeAllChunksVisible,
    MakeAllChunksInvisible,
    Visualize,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    let model = Model {
        img: StaticImage::default(),
        config: VisConfig::default(),
        refs: Refs::default(),
    };

    orders.after_next_render(|_| Msg::Visualize);

    model
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::InputFileChanged => {
            let input_file = model.refs.input_file.get().unwrap();
            let files = input_file.files().unwrap();
            let files = gloo_file::FileList::from(files);
            if files.is_empty() {
                return;
            }

            orders.perform_cmd(async move {
                let file = &files[0];

                match StaticImage::from_blob(file.name(), file).await {
                    Ok(img) => {
                        log!("loaded image '{}'", file.name());
                        Some(Msg::UpdateImage(img))
                    }
                    Err(e) => {
                        log!("cannot load image '{}': {}", file.name(), e);
                        None
                    }
                }
            });
        }

        Msg::UpdateImage(img) => {
            model.img = img;

            orders.after_next_render(|_| Msg::Visualize);
        }

        Msg::ToggleChunkVisibility(chunk) => {
            model.config.toggle_visibility(chunk);

            orders.after_next_render(|_| Msg::Visualize);
        }

        Msg::MakeAllChunksVisible => {
            model.config.make_all_visible();

            orders.after_next_render(|_| Msg::Visualize);
        }

        Msg::MakeAllChunksInvisible => {
            model.config.make_all_invisible();

            orders.after_next_render(|_| Msg::Visualize);
        }

        Msg::Visualize => {
            draw_vis(model);
        }
    }
}

fn draw_vis(model: &Model) {
    let img_vis = visualize(&model.img, &model.config);
    let image_data = util::create_image_data(&img_vis).unwrap();

    let canvas = model.refs.canvas.get().unwrap();
    let ctx = canvas_context_2d(&canvas);

    ctx.put_image_data(&image_data, 0., 0.).unwrap();
}

fn view(model: &Model) -> Node<Msg> {
    div![view_header(model), view_sidebar(model), view_vis(model)]
}

fn view_header(model: &Model) -> Vec<Node<Msg>> {
    nodes![
        div![
            label![
                attrs! {
                    At::For => "input-file",
                },
                "Open image file: ",
            ],
            input![
                el_ref(&model.refs.input_file),
                attrs! {
                    At::Id => "input-file",
                    At::Type => "file",
                },
                ev(Ev::Change, |_| Msg::InputFileChanged),
            ],
        ],
        hr![],
    ]
}

fn view_sidebar(model: &Model) -> Node<Msg> {
    let table_rows: Vec<_> = QoiChunk::iter()
        .map(|chunk| {
            let idx = chunk as usize;
            let id_str = format!("checkbox-visible-{}", idx);
            let count = model.img.histogram()[idx];
            let percent = 100. * (count as f64) / (model.img.pixel_count() as f64);
            let [r, g, b] = color_of_chunk(chunk);
            let color_str = format!("rgb({},{},{})", r, g, b);
            tr![
                td![input![
                    id!(&id_str),
                    attrs! {
                        At::Type => "checkbox",
                    },
                    IF!(model.config.is_visible(chunk) => attrs! {
                        At::Checked => "",
                    }),
                    ev(Ev::Change, move |_| Msg::ToggleChunkVisibility(chunk)),
                ]],
                td![
                    style! {
                        St::TextAlign => "right",
                    },
                    label![attrs! {At::For => &id_str}, format!("{: >5.2} %", percent)]
                ],
                td![label![
                    attrs! {At::For => &id_str},
                    svg![
                        attrs! {
                            At::Width => px(24),
                            At::Height => px(24),
                            At::ViewBox => "0 0 24 24",
                        },
                        rect![attrs! {
                            At::Width => 24,
                            At::Height => 24,
                            At::Stroke => "black",
                            At::Fill => color_str,
                        }],
                    ]
                ]],
                td![label![attrs! {At::For => &id_str}, chunk.name()]],
            ]
        })
        .collect();

    div![
        id!("sidebar"),
        div![
            button!["check all", ev(Ev::Click, |_| Msg::MakeAllChunksVisible)],
            " ",
            button![
                "uncheck all",
                ev(Ev::Click, |_| Msg::MakeAllChunksInvisible),
            ],
        ],
        table![tbody![table_rows]],
        view_sidebar_info(model),
    ]
}

fn view_sidebar_info(model: &Model) -> Node<Msg> {
    div![
        div![model.img.name()],
        table![
            tr![td!["Original size"], td![model.img.filesize_orig()]],
            tr![td!["QOI size"], td![model.img.filesize_qoi()]],
        ],
    ]
}

fn view_vis(model: &Model) -> Node<Msg> {
    div![
        id!("vis"),
        div![
            p!["Original image:"],
            img![attrs! {
                At::Src => model.img.url(),
            }],
        ],
        div![
            p!["Visualization:"],
            canvas![
                el_ref(&model.refs.canvas),
                attrs! {
                    At::Width => px(model.img.width()),
                    At::Height => px(model.img.height()),
                }
            ],
        ],
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    App::start("app", init, update, view);
}
