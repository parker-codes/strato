#![allow(non_snake_case)]

use dioxus::prelude::*;

fn main() {
    dioxus_web::launch(App);
}

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            class: "p-4 w-full h-full flex flex-col justify-center items-center sm:gap-y-12 dark:bg-slate-900",

            Appropriation {}
        }
    })
}

#[inline_props]
fn Appropriation(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            class: "group fixed bottom-10 right-0 left-2",
            div {
                class: "transition-opacity group-hover:opacity-0 absolute right-6",
                Heart {}
            }
            div {
                class: "transition-opacity opacity-0 group-hover:opacity-100 absolute right-2 xs:right-6 text-center text-xs text-gray-700 dark:text-gray-400",
                "Made with "
                Heart {}
                " by Parker McMullin (aka. "
                a {
                    href: "https://twitter.com/parker_codes",
                    target: "_blank",
                    "@parker_codes"
                }
                ")"
            }
        }
    })
}

#[inline_props]
fn Heart(cx: Scope) -> Element {
    cx.render(rsx! {
        span {
            class: "text-red-500 dark:text-red-700",
            "❤"
        }
    })
}
