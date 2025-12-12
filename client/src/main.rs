#![allow(non_snake_case)]

use leptos::prelude::*;
use std::rc::Rc;
use strato::card::{CardValue, CellView};
use strato::game::{
    EndAction, GameOptions, GameSnapshot, GameState, PlayerView, StartAction, StratoGame,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (game, set_game) = signal(bootstrap_game());
    let (mode, set_mode) = signal(TurnMode::Swap);
    let (message, set_message) = signal("Welcome to Strato!".to_string());

    let snapshot = Memo::new(move |_| game.get().snapshot());

    let restart = move |_| {
        set_game.set(bootstrap_game());
        set_mode.set(TurnMode::Swap);
        set_message.set("New game started.".to_string());
    };

    let start_from_deck = move |_| {
        if let Some(player_id) = current_player_id(snapshot()) {
            set_game.update(|g| {
                match g.start_player_turn(player_id.clone(), StartAction::DrawFromDeck) {
                    Ok(_) => set_message.set("Drew from deck.".to_string()),
                    Err(err) => set_message.set(err.to_string()),
                }
            });
        }
    };

    let start_from_discard = move |_| {
        if let Some(player_id) = current_player_id(snapshot()) {
            set_game.update(|g| {
                match g.start_player_turn(player_id.clone(), StartAction::TakeFromDiscardPile) {
                    Ok(_) => set_message.set("Took from discard pile.".to_string()),
                    Err(err) => set_message.set(err.to_string()),
                }
            });
        }
    };

    let handle_cell_click = Rc::new(move |player_idx: usize, row: usize, column: usize| {
        set_game.update(|g| match g.state {
            GameState::DetermineFirstPlayer => {
                let Some(player_id) = g.context.players.get(player_idx).map(|p| p.id()) else {
                    return;
                };

                if let Err(err) = g.player_flip_to_determine_who_is_first(player_id, row, column) {
                    set_message.set(err.to_string());
                }
            }
            GameState::Active | GameState::LastRound => {
                if Some(player_idx) != g.context.current_player_idx {
                    set_message.set("It's not this player's turn.".to_string());
                    return;
                }

                let Some(player_id) = g.context.players.get(player_idx).map(|p| p.id()) else {
                    return;
                };

                let Some(card_in_hand) = g.context.players[player_idx].holding() else {
                    set_message.set("Start your turn by drawing a card.".to_string());
                    return;
                };

                let action = match mode.get() {
                    TurnMode::Swap => EndAction::Swap { row, column },
                    TurnMode::Flip => EndAction::Flip { row, column },
                };

                match g.end_player_turn(player_id, action) {
                    Ok(_) => set_message.set(format!(
                        "Placed {} and advanced the turn.",
                        i32::from(card_in_hand.face_value().unwrap())
                    )),
                    Err(err) => set_message.set(err.to_string()),
                }
            }
            GameState::Startup | GameState::WaitingForPlayers | GameState::Ended => {}
        });
    });

    view! {
        <main class="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 text-slate-100 px-4 pb-12">
            <header class="max-w-6xl mx-auto flex items-center justify-between py-6">
                <div>
                    <p class="text-sm uppercase tracking-wide text-slate-400">
                        "Skyjo-inspired"
                    </p>
                    <h1 class="text-4xl font-semibold">
                        "Strato"
                    </h1>
                </div>
                <button class="px-4 py-2 rounded-lg bg-indigo-500 hover:bg-indigo-400 text-white text-sm font-semibold" on:click=restart>
                    "Reset game"
                </button>
            </header>

            <section class="max-w-6xl mx-auto grid gap-6 md:grid-cols-3">
                <div class="md:col-span-2 space-y-4">
                    <GameMeta snapshot={snapshot} />
                    <PlayersGrid snapshot={snapshot} on_cell_click={handle_cell_click.clone()} />
                </div>
                <Sidebar
                    snapshot={snapshot}
                    mode={mode}
                    set_mode={set_mode}
                    on_draw_deck={start_from_deck}
                    on_draw_discard={start_from_discard}
                    message={message}
                />
            </section>
        </main>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TurnMode {
    Swap,
    Flip,
}

fn bootstrap_game() -> StratoGame {
    let mut game = StratoGame::new();
    let _ = game.add_player("You");
    let _ = game.add_player("Nova");
    let _ = game.add_player("Kai");

    let options = GameOptions {
        first_player_idx: Some(0),
    };

    let _ = game.start_with_options(options);

    game
}

fn current_player_id(snapshot: GameSnapshot) -> Option<String> {
    snapshot
        .current_player_idx
        .and_then(|idx| snapshot.players.get(idx).map(|p| p.id.clone()))
}

#[component]
fn GameMeta(snapshot: Memo<GameSnapshot>) -> impl IntoView {
    let status = move || snapshot().state.clone();
    let round = move || snapshot().round + 1;
    let discard_top = move || snapshot().discard_top;
    let deck_size = move || snapshot().deck_size;
    let discard_size = move || snapshot().discard_size;

    view! {
        <section class="rounded-xl bg-slate-800/80 shadow-lg shadow-slate-950/40 border border-slate-700/50 p-4 flex flex-wrap items-center gap-4">
            <div class="flex-1">
                <p class="text-xs uppercase tracking-wide text-slate-400">
                    "State"
                </p>
                <h2 class="text-xl font-semibold">
                    move || format!("{status:?}")
                </h2>
            </div>
            <div class="w-[1px] self-stretch bg-slate-700/50"></div>
            <div class="flex flex-col items-center justify-center">
                <p class="text-xs uppercase tracking-wide text-slate-400">
                    "Round"
                </p>
                <p class="text-3xl font-bold">
                    move || round().to_string()
                </p>
            </div>
            <div class="grid grid-cols-2 gap-3">
                <div class="p-3 rounded-lg bg-slate-900/60 border border-slate-700/50">
                    <p class="text-xs uppercase tracking-wide text-slate-400">
                        "Deck"
                    </p>
                    <p class="text-lg font-semibold">
                        move || format!("{} cards", deck_size())
                    </p>
                </div>
                <div class="p-3 rounded-lg bg-slate-900/60 border border-slate-700/50">
                    <p class="text-xs uppercase tracking-wide text-slate-400">
                        "Discard"
                    </p>
                    <p class="text-lg font-semibold">
                        move || format!("{} cards", discard_size())
                    </p>
                    <div class="mt-2">
                        <CardView slot={discard_top().map(CellView::Revealed)} />
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn PlayersGrid(
    snapshot: Memo<GameSnapshot>,
    on_cell_click: Rc<dyn Fn(usize, usize, usize)>,
) -> impl IntoView {
    let players = move || snapshot().players;
    view! {
        <section class="grid gap-4 sm:grid-cols-2">
            <For
                each={players}
                key={|(player): &PlayerView| player.id.clone()}
                children={move |player: PlayerView| {
                    let idx = snapshot()
                        .players
                        .iter()
                        .position(|p| p.id == player.id)
                        .unwrap_or(0);
                    view! {
                        <PlayerPanel
                            player={player}
                            is_current={move || snapshot().current_player_idx == Some(idx)}
                            on_cell_click={on_cell_click.clone()}
                            player_idx={idx}
                        />
                    }
                }}
            />
        </section>
    }
}

#[component]
fn PlayerPanel(
    player: PlayerView,
    is_current: impl Fn() -> bool + 'static,
    on_cell_click: Rc<dyn Fn(usize, usize, usize)>,
    player_idx: usize,
) -> impl IntoView {
    let name = player.name.clone();
    let holding = player.holding;
    let spread = player.spread.clone();
    let flipped = player.flipped;
    let score = player.score;

    let header_class = move || match is_current() {
        true => "bg-indigo-500 text-white",
        false => "bg-slate-700/80 text-slate-100",
    };

    view! {
        <article class="rounded-xl overflow-hidden border border-slate-700/60 bg-slate-800/60 shadow-lg shadow-slate-950/30">
            <div class={move || format!("flex items-center justify-between px-4 py-2 text-sm font-semibold {}", header_class())}>
                <span>name.clone()</span>
                <span class="text-xs">
                    move || format!("{} flipped", flipped)
                </span>
            </div>
            <div class="p-4 space-y-3">
                <div class="flex items-center gap-2 text-sm text-slate-300">
                    <span>"Score:"</span>
                    <span class="text-lg font-semibold text-white">score</span>
                    <span class="ml-auto text-xs uppercase tracking-wide text-slate-400">
                        if holding.is_some() { "Holding" } else { "Hand empty" }
                    </span>
                </div>
                <div class="grid grid-rows-3 grid-cols-4 gap-3">
                    {spread.iter().enumerate().map(|(row_idx, row)| {
                        row.iter().enumerate().map(move |(col_idx, slot)| {
                            let slot = *slot;
                            let click = on_cell_click.clone();
                            let player_idx = player_idx;
                            view! {
                                <button
                                    class="relative aspect-[2.5/3.5] rounded-lg overflow-hidden shadow-md border border-slate-700/70 hover:border-indigo-400 transition"
                                    on:click={move |_| click(player_idx, row_idx, col_idx)}
                                >
                                    <CardView slot={Some(slot)} />
                                </button>
                            }
                        }).collect_view()
                    }).collect_view()}
                </div>
                if let Some(value) = holding {
                    view! {
                        <div class="flex items-center gap-2 text-sm text-slate-300">
                            <span>"In hand:"</span>
                            <CardView slot={Some(CellView::Revealed(value))} />
                        </div>
                    }
                }
            </div>
        </article>
    }
}

#[component]
fn Sidebar(
    snapshot: Memo<GameSnapshot>,
    mode: ReadSignal<TurnMode>,
    set_mode: WriteSignal<TurnMode>,
    on_draw_deck: impl Fn(MouseEvent) + 'static,
    on_draw_discard: impl Fn(MouseEvent) + 'static,
    message: ReadSignal<String>,
) -> impl IntoView {
    let deck_size = move || snapshot().deck_size;
    let discard_top = move || snapshot().discard_top;

    view! {
        <aside class="h-full space-y-4">
            <div class="rounded-xl bg-slate-800/80 border border-slate-700/70 p-4 shadow-md shadow-slate-950/30">
                <h3 class="text-lg font-semibold mb-2">"Turn actions"</h3>
                <div class="space-y-2">
                    <button class="w-full px-3 py-2 rounded-lg bg-indigo-500 hover:bg-indigo-400 text-white text-sm font-semibold disabled:opacity-60" on:click={on_draw_deck} disabled={move || snapshot().state == GameState::Ended}>
                        "Draw from deck (" {deck_size} ")"
                    </button>
                    <button class="w-full px-3 py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-white text-sm font-semibold disabled:opacity-60" on:click={on_draw_discard} disabled={move || discard_top().is_none()}>
                        "Take from discard"
                    </button>
                </div>
                <div class="pt-3 border-t border-slate-700/60 mt-3 space-y-2">
                    <p class="text-xs uppercase tracking-wide text-slate-400">"Placement mode"</p>
                    <div class="flex gap-2">
                        <ToggleButton label="Swap" active={move || mode.get() == TurnMode::Swap} on_click={move |_| set_mode.set(TurnMode::Swap)} />
                        <ToggleButton label="Flip" active={move || mode.get() == TurnMode::Flip} on_click={move |_| set_mode.set(TurnMode::Flip)} />
                    </div>
                </div>
            </div>
            <div class="rounded-xl bg-slate-900/80 border border-slate-800 p-4 text-sm shadow-inner shadow-slate-950/30 min-h-[120px]">
                <p class="text-xs uppercase tracking-wide text-slate-400">"Game log"</p>
                <p class="mt-2 text-slate-200">message</p>
            </div>
        </aside>
    }
}

#[component]
fn ToggleButton(
    label: &'static str,
    active: impl Fn() -> bool + 'static,
    on_click: impl Fn(MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <button
            class={move || {
                let base = "flex-1 px-3 py-2 rounded-lg text-sm font-semibold border border-slate-700";
                if active() {
                    format!("{} bg-indigo-500 text-white", base)
                } else {
                    format!("{} bg-slate-800 text-slate-200", base)
                }
            }}
            on:click={on_click}
        >
            label
        </button>
    }
}

#[component]
fn CardView(slot: Option<CellView>) -> impl IntoView {
    let content = move || match slot {
        None => view! { <div class="h-full w-full bg-slate-900/50"></div> },
        Some(CellView::Empty) => view! { <div class="h-full w-full bg-slate-900/50"></div> },
        Some(CellView::Hidden) => view! { <div><CardBack /></div> },
        Some(CellView::Revealed(value)) => view! { <div><CardFace value={value} /></div> },
    };

    view! {
        <div class="relative h-full w-full">{content()}</div>
    }
}

const HEX_PATTERN: &str = "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='28' height='49' viewBox='0 0 28 49'%3E%3Cg fill-rule='evenodd'%3E%3Cg id='hexagons' fill='%23ffffff' fill-opacity='0.4' fill-rule='nonzero'%3E%3Cpath d='M13.99 9.25l13 7.5v15l-13 7.5L1 31.75v-15l12.99-7.5zM3 17.9v12.7l10.99 6.34 11-6.35V17.9l-11-6.34L3 17.9zM0 15l12.98-7.5V0h-2v6.35L0 12.69v2.3zm0 18.5L12.98 41v8h-2v-6.85L0 35.81v-2.3zM15 0v7.5L27.99 15H28v-2.31h-.01L17 6.35V0h-2zm0 49v-8l12.99-7.5H28v2.31h-.01L17 42.15V49h-2z'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E\")";

#[component]
fn CardFace(value: CardValue) -> impl IntoView {
    let value_display = i32::from(value).to_string();
    let face_color_class = get_face_color_class(value);
    let underline_class = get_underline_class(value);

    view! {
        <div class=move || format!("h-full {} relative", face_color_class) style=format!("background-image: {}", HEX_PATTERN)>
            <div class="absolute inset-[10%]">
                <svg class="h-full w-full" viewBox="0 0 103 103">
                    <polygon class="stroke-0 fill-white opacity-40" points="50 3,100 28,100 75, 50 100,3 75,3 25" />
                </svg>
            </div>
            <span class={move || format!("absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-5xl font-bold text-black {}", underline_class)} style="text-shadow: 3px 3px 3px white;">
                {value_display}
            </span>
        </div>
    }
}

#[component]
fn CardBack() -> impl IntoView {
    view! {
        <div class="h-full bg-slate-900 relative" style=format!("background-image: {}", HEX_PATTERN)>
            <div class="absolute inset-[5%] border-4 border-white opacity-40"></div>
            <span class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-5xl font-bold text-white">
                "S"
            </span>
        </div>
    }
}

fn get_face_color_class(value: CardValue) -> String {
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
