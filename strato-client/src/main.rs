#![allow(non_snake_case)]

use dioxus::prelude::*;
use std::default::Default;
use strato_core::card::{CardValue, Deck, PlayerSpread};
use web_sys::console;

fn main() {
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    let mut deck = Deck::default();
    let mut spread = PlayerSpread::new();
    spread.place_at(deck.draw().unwrap(), 0, 0).unwrap();
    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 0, 1).unwrap();
    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 0, 2).unwrap();
    let mut flipped_card = deck.draw().unwrap();
    flipped_card.flip();
    spread.place_at(flipped_card, 2, 1).unwrap();

    cx.render(rsx! {
        div { "Hello, world!" },
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
fn Card(cx: Scope<CardProps>, #[props(!optional)] value: Option<CardValue>) -> Element {
    let value_display = match *value {
        Some(value) => (value as i32).to_string(),
        None => "(hidden)".to_string(),
    };
    // let face_color = get_face_color_class(value.unwrap_or(CardValue::NegativeTwo));

    cx.render(rsx! {
        div {
            class: "border-2 border-black aspect-[2.5/3.5] grid place-content-center",
            "{value_display}"
        }
    })
}

fn get_face_color_class(value: CardValue) -> String {
    // TODO: Somehow cards are starting at 13 instead of 12...
    // I need to check the tests output to see if this is true everywhere
    console::log_1(&format!("{}", value as i32).into());
    match value as i32 {
        -2..=-1 => String::from("bg-indigo-400"),
        0 => String::from("bg-sky-400"),
        1..=4 => String::from("bg-green-400"),
        5..=8 => String::from("bg-yellow-500"),
        9..=12 => String::from("bg-red-400"),
        _ => unreachable!(),
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
