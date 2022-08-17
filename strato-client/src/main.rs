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
    if let Some(value) = *value {
        // face of card

        let value_display = (value as i32).to_string();
        let face_color = get_face_color_class(value);

        return cx.render(rsx! {
            div {
                class: "border-2 border-black aspect-[2.5/3.5] {face_color} grid place-content-center",
                "{value_display}"
            }
        });
    } else {
        // back of card

        return cx.render(rsx! {
            div {
                class: "border-2 border-black aspect-[2.5/3.5] grid place-content-center",
                "(hidden)"
            }
        });
    }
}

fn get_face_color_class(value: CardValue) -> String {
    // TODO: Somehow cards are starting at 13 instead of 12...
    // I need to check the tests output to see if this is true everywhere
    console::log_1(&format!("as i32: {}", value as i32).into());
    console::log_1(&format!("i32::from: {}", i32::from(value)).into());
    match i32::from(value) {
        -2..=-1 => String::from("bg-indigo-300"),
        0 => String::from("bg-sky-300"),
        1..=4 => String::from("bg-green-300"),
        5..=8 => String::from("bg-yellow-300"),
        9..=12 => String::from("bg-red-300"),
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
