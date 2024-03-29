#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::default::Default;
use strato::card::{CardValue, Deck, PlayerSpread};
use web_sys::console;

fn main() {
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    let mut deck = Deck::default();
    let mut spread = PlayerSpread::new();

    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 0, 0).unwrap();

    // 11-10
    deck.draw().unwrap();
    deck.draw().unwrap();

    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 0, 1).unwrap();

    deck.draw().unwrap();
    // let mut flipped_card = deck.draw().unwrap();
    // flipped_card.flip();
    // spread.place_at(flipped_card, 0, 1).unwrap();

    deck.draw().unwrap();
    // let mut flipped_card = deck.draw().unwrap();
    // flipped_card.flip();
    // spread.place_at(flipped_card, 0, 2).unwrap();

    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 0, 3).unwrap();

    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 1, 0).unwrap();

    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 1, 1).unwrap();

    deck.draw().unwrap();
    // let mut flipped_card = deck.draw().unwrap();
    // flipped_card.flip();
    // spread.place_at(flipped_card, 1, 2).unwrap();

    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 1, 3).unwrap();

    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 2, 0).unwrap();

    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 2, 1).unwrap();

    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 2, 2).unwrap();

    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 2, 3).unwrap();

    cx.render(rsx! {
        div {
            class: "text-white text-2xl",
            "Strato!"
        },

        // TODO: create component for PlayerSpread
        div {
            class: "w-full max-w-xl grid grid-rows-3 grid-cols-4 justify-center gap-4",
            {spread.view().iter().map(|row| rsx! {
                {row.iter().map(|card_value| rsx! {
                    Card { value: *card_value }
                })}
            })}
        },

        Heart {},
    })
}

#[inline_props]
fn Card(cx: Scope, #[props(!optional)] value: Option<CardValue>) -> Element {
    return cx.render(rsx! {
        div {
            class: "relative aspect-[2.5/3.5] bg-white rounded-md",

            div {
                class: "absolute inset-1",

                if let Some(value) = *value {
                    rsx!(FaceOfCard { value: value })
                } else {
                    rsx!(BackOfCard {})
                }
            }
        }
    });
}

const HEX_PATTERN: &'static str = "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='28' height='49' viewBox='0 0 28 49'%3E%3Cg fill-rule='evenodd'%3E%3Cg id='hexagons' fill='%23ffffff' fill-opacity='0.4' fill-rule='nonzero'%3E%3Cpath d='M13.99 9.25l13 7.5v15l-13 7.5L1 31.75v-15l12.99-7.5zM3 17.9v12.7l10.99 6.34 11-6.35V17.9l-11-6.34L3 17.9zM0 15l12.98-7.5V0h-2v6.35L0 12.69v2.3zm0 18.5L12.98 41v8h-2v-6.85L0 35.81v-2.3zM15 0v7.5L27.99 15H28v-2.31h-.01L17 6.35V0h-2zm0 49v-8l12.99-7.5H28v2.31h-.01L17 42.15V49h-2z'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E\")";

#[inline_props]
fn FaceOfCard(cx: Scope, value: CardValue) -> Element {
    let value_display = i32::from(*value).to_string();
    let face_color_class = get_face_color_class(*value);
    let underline_class = get_underline_class(*value);

    return cx.render(rsx! {
        div {
            class: "h-full {face_color_class}",
            background_image: "{HEX_PATTERN}",

            div {
                class: "absolute inset-[10%]",
                svg {
                    class: "h-full w-full",
                    view_box: "0 0 103 103",

                    polygon {
                        class: "stroke-0 fill-white opacity-40",
                        points: "50 3,100 28,100 75, 50 100,3 75,3 25",
                    },
                },
            },

            span {
                class: "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-5xl font-bold text-black {underline_class}",
                text_shadow: "3px 3px 3px white",
                "{value_display}"
            }
        }
    });
}

#[inline_props]
fn BackOfCard(cx: Scope) -> Element {
    return cx.render(rsx! {
        div {
            class: "h-full bg-slate-900",
            background_image: "{HEX_PATTERN}",

            div {
                class: "absolute inset-[5%] border-4 border-white opacity-40",
            },

            span {
                class: "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-5xl font-bold text-white",
                "S"
            }
        }
    });
}

fn get_face_color_class(value: CardValue) -> String {
    console::log_1(&format!("{:?} -> i32::from: {}", value, i32::from(value)).into());
    match i32::from(value) {
        -2..=-1 => String::from("bg-indigo-500"),
        0 => String::from("bg-sky-400"),
        1..=4 => String::from("bg-green-400"),
        5..=8 => String::from("bg-yellow-300"),
        9..=12 => String::from("bg-red-500"),
        _ => unreachable!(),
    }
}

fn get_underline_class(value: CardValue) -> String {
    match value {
        CardValue::Six | CardValue::Nine => String::from("underline"),
        _ => String::from(""),
    }
}

#[inline_props]
fn Heart(cx: Scope) -> Element {
    cx.render(rsx! {
        span {
            class: "text-blue-500 dark:text-blue-700",
            "❤"
        }
    })
}
