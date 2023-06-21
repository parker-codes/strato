use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use strato::{
    self,
    card::Deck,
    game::{
        AddPlayerAction, GameEvent::*, GameOptions, GameStartupError, GameState, PlayerTurnError,
        StratoGame, SubscriberEvent,
    },
    player::{generate_player_id, EndAction, StartAction},
};

fn start_game_with_order() -> (StratoGame<'static>, String, String) {
    let mut game = StratoGame::new();
    let player_1_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_1_id }));
    let player_2_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_2_id }));
    game.start_with_options(GameOptions {
        first_player_idx: Some(0),
    })
    .unwrap();
    (game, player_1_id, player_2_id)
}

#[test]
fn a_game_can_be_initialized() {
    let game = StratoGame::new();
    assert_eq!(game.state, GameState::WaitingForPlayers);
    assert_eq!(game.context.deck.size(), Deck::FULL_SIZE);
}

#[test]
fn players_can_be_added() {
    let mut game = StratoGame::new();
    let player_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_id }));
    assert_eq!(game.state, GameState::WaitingForPlayers);
    assert_eq!(game.context.players.len(), 1);
}

#[test]
fn a_game_can_be_started() {
    let mut game = StratoGame::new();
    let player_1_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_1_id }));
    let player_2_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_2_id }));
    let result = game.start();
    assert!(result.is_ok());
    assert_eq!(game.state, GameState::DetermineFirstPlayer);
}

#[test]
fn a_game_can_be_started_with_specific_start_player() {
    let previous_winner_idx = 1;

    let mut game = StratoGame::new();
    let player_1_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_1_id }));
    let player_2_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_2_id }));
    let result = game.start_with_options(GameOptions {
        first_player_idx: Some(previous_winner_idx),
    });
    assert!(result.is_ok());
    assert_eq!(game.state, GameState::Active);
}

#[test]
fn cant_start_without_players() {
    let mut game = StratoGame::new();
    let result = game.start();
    assert_eq!(result.unwrap_err(), GameStartupError::NotEnoughPlayers);
    assert_eq!(game.state, GameState::WaitingForPlayers);
}

#[test]
fn a_started_game_deals_cards_to_players() {
    let (game, player_1_id, _) = start_game_with_order();
    let player = game.get_player(&player_1_id).unwrap();

    assert_eq!(
        player
            .spread
            .view()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .len(),
        12
    );
    let cards_used = (12 * 2) /* for 2 players */ + 1 /* for discard init */;
    assert_eq!(game.context.deck.size(), Deck::FULL_SIZE - cards_used);
}

#[test]
fn starting_multiple_times_is_inconsequential() {
    let (mut game, _, _) = start_game_with_order();
    let deck_snapshot = game.context.deck.clone();
    assert!(game.start().is_err());
    assert_eq!(
        game.start().unwrap_err(),
        GameStartupError::GameAlreadyStarted
    );
    assert_eq!(game.state, GameState::Active);
    assert_eq!(deck_snapshot, game.context.deck);
}

#[test]
fn can_list_players() {
    let mut game = StratoGame::new();
    let player_1_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_1_id }));
    let player_2_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_2_id }));
    let player_1 = game.get_player(player_1_id).unwrap();
    let player_2 = game.get_player(player_2_id).unwrap();
    assert!(game.list_players().iter().eq(vec![player_1, player_2]));
}

#[test]
fn cant_change_players_after_game_starts() {
    let mut game = StratoGame::new();
    let player_1_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_1_id }));
    let player_2_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_2_id }));
    game.start_with_options(GameOptions {
        first_player_idx: Some(0),
    })
    .unwrap();
    assert_eq!(game.state, GameState::Active);

    let player_3_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_3_id }));
    assert_eq!(game.list_players().len(), 2);
}

#[test]
fn the_first_turn_can_take_from_discard_pile() {
    let (mut game, player_1_id, _) = start_game_with_order();
    let turn = game.start_player_turn(&player_1_id, StartAction::TakeFromDiscardPile);
    assert!(turn.is_ok());
}

#[test]
fn a_player_can_draw_and_flip() {
    let (mut game, player_1_id, _) = start_game_with_order();
    game.start_player_turn(&player_1_id, StartAction::DrawFromDeck)
        .expect("Couldn't start turn");
    assert!(game.get_player(&player_1_id).unwrap().holding().is_some());
    game.end_player_turn(&player_1_id, EndAction::Flip { row: 1, column: 2 })
        .expect("Couldn't end turn");
    assert!(game.get_player(&player_1_id).unwrap().holding().is_none());
    assert_eq!(game.context.discard_pile.size(), 2); // discard init contains 1 already
}

#[test]
fn a_player_can_take_and_swap() {
    let (mut game, player_1_id, _) = start_game_with_order();
    game.start_player_turn(&player_1_id, StartAction::TakeFromDiscardPile)
        .expect("Couldn't start turn");
    assert!(game.get_player(&player_1_id).unwrap().holding().is_some());
    game.end_player_turn(&player_1_id, EndAction::Swap { row: 2, column: 2 })
        .expect("Couldn't end turn");
    assert!(game.get_player(&player_1_id).unwrap().holding().is_none());
    assert_eq!(game.context.discard_pile.size(), 1); // discard init contains 1 already
}

#[test]
fn cant_flip_same_card_twice() {
    let (mut game, player_1_id, player_2_id) = start_game_with_order();

    const ROW: usize = 0;
    const COLUMN: usize = 1;

    // First turn
    game.start_player_turn(&player_1_id, StartAction::DrawFromDeck)
        .expect("Couldn't start Player 1's turn");
    game.end_player_turn(
        &player_1_id,
        EndAction::Flip {
            row: ROW,
            column: COLUMN,
        },
    )
    .expect("Couldn't end Player 1's turn");

    // Other player's turn
    game.start_player_turn(&player_2_id, StartAction::DrawFromDeck)
        .expect("Couldn't start Player 2's turn");
    game.end_player_turn(
        &player_2_id,
        EndAction::Flip {
            row: ROW,
            column: COLUMN,
        },
    )
    .expect("Couldn't end Player 2's turn");

    // Second turn
    game.start_player_turn(&player_1_id, StartAction::DrawFromDeck)
        .expect("Couldn't start Player 1's second turn");
    let turn = game.end_player_turn(
        &player_1_id,
        EndAction::Flip {
            row: ROW,
            column: COLUMN,
        },
    );
    assert!(turn.is_err());
}

#[test]
fn cant_start_turn_twice() {
    let (mut game, player_1_id, _) = start_game_with_order();

    game.start_player_turn(&player_1_id, StartAction::DrawFromDeck)
        .expect("Couldn't start Player 1's turn");
    let result = game.start_player_turn(&player_1_id, StartAction::DrawFromDeck);

    assert_eq!(result.unwrap_err(), PlayerTurnError::TurnAlreadyStarted);
}

#[test]
fn multiple_players_session_1() {
    let mut game = StratoGame::new();
    let cassie_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &cassie_id }));
    let james_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &james_id }));
    game.start_with_options(GameOptions {
        first_player_idx: Some(0),
    })
    .unwrap();

    assert_eq!(game.state, GameState::Active);

    // Cassie first
    game.start_player_turn(&cassie_id, StartAction::DrawFromDeck)
        .expect("Couldn't start Cassie's turn");
    game.end_player_turn(&cassie_id, EndAction::Flip { row: 1, column: 2 })
        .expect("Couldn't end Cassie's turn");
    assert_eq!(game.context.discard_pile.size(), 2);

    // James next
    game.start_player_turn(&james_id, StartAction::TakeFromDiscardPile)
        .expect("Couldn't start James's turn");
    game.end_player_turn(&james_id, EndAction::Swap { row: 2, column: 2 })
        .expect("Couldn't end James's turn");
    assert_eq!(game.context.discard_pile.size(), 2); // hasn't changed because this was taken from discard pile

    // Then Cassie again
    game.start_player_turn(&cassie_id, StartAction::DrawFromDeck)
        .expect("Couldn't start Cassie's 2nd turn");
    game.end_player_turn(&cassie_id, EndAction::Swap { row: 2, column: 3 })
        .expect("Couldn't end Cassie's 2nd turn");
    assert_eq!(game.context.discard_pile.size(), 3);

    // Then James again
    game.start_player_turn(&james_id, StartAction::DrawFromDeck)
        .expect("Couldn't start James's 2nd turn");
    game.end_player_turn(&james_id, EndAction::Flip { row: 0, column: 0 })
        .expect("Couldn't end James's 2nd turn");
    assert_eq!(game.context.discard_pile.size(), 4);

    let cassie = &game.get_player(&cassie_id).unwrap();
    let flipped_over_in_spread = cassie
        .spread
        .view()
        .into_iter()
        .flatten()
        .filter(|card| card.is_some())
        .collect::<Vec<_>>()
        .len();
    assert_eq!(flipped_over_in_spread, 2);
}

#[test]
fn can_subscribe_to_changes() {
    let mut game = StratoGame::new();
    let state_change_triggered = Arc::new(AtomicBool::new(false));
    let context_change_triggered = Arc::new(AtomicBool::new(false));

    game.subscribe({
        let state_change_triggered = state_change_triggered.clone();
        let context_change_triggered = context_change_triggered.clone();

        move |event| match event {
            SubscriberEvent::StateChanged(_) => {
                state_change_triggered.store(true, Ordering::Relaxed)
            }
            SubscriberEvent::ContextChanged(_) => {
                context_change_triggered.store(true, Ordering::Relaxed)
            }
        }
    });

    let player_1_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &player_1_id }));
    // not tracked
    game.send(AddPlayer(AddPlayerAction {
        id: &generate_player_id(),
    }));
    game.start_with_options(GameOptions {
        first_player_idx: Some(0),
    })
    .unwrap();

    game.start_player_turn(&player_1_id, StartAction::DrawFromDeck)
        .expect("Couldn't start turn");

    assert_eq!(state_change_triggered.load(Ordering::Relaxed), true);
    assert_eq!(context_change_triggered.load(Ordering::Relaxed), true);
}

#[test]
fn can_flip_to_determine_who_is_first() {
    let mut game = StratoGame::new();
    let cassie_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &cassie_id }));
    let james_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &james_id }));
    game.start().unwrap();

    assert_eq!(game.state, GameState::DetermineFirstPlayer);

    game.player_flip_to_determine_who_is_first(&cassie_id, 0, 0)
        .unwrap();
    game.player_flip_to_determine_who_is_first(&cassie_id, 1, 0)
        .unwrap();

    assert_eq!(game.state, GameState::DetermineFirstPlayer);

    game.player_flip_to_determine_who_is_first(&james_id, 2, 1)
        .unwrap();
    game.player_flip_to_determine_who_is_first(&james_id, 1, 3)
        .unwrap();

    assert_eq!(game.state, GameState::Active);

    let result = game.player_flip_to_determine_who_is_first(&cassie_id, 2, 0);
    assert_eq!(
        result.unwrap_err(),
        PlayerTurnError::NotDeterminingFirstPlayer
    );

    let current_player_idx = game.context.current_player_idx;
    assert!(current_player_idx.is_some());
    assert!((0..=2).contains(&current_player_idx.unwrap()));
}

#[test]
fn cant_flip_too_many_cards_to_determine_first_player() {
    let mut game = StratoGame::new();
    let cassie_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &cassie_id }));
    // untracked
    game.send(AddPlayer(AddPlayerAction {
        id: &generate_player_id(),
    }));
    game.start().unwrap();

    assert_eq!(game.state, GameState::DetermineFirstPlayer);

    game.player_flip_to_determine_who_is_first(&cassie_id, 0, 0)
        .unwrap();
    game.player_flip_to_determine_who_is_first(&cassie_id, 1, 0)
        .unwrap();
    let result = game.player_flip_to_determine_who_is_first(&cassie_id, 2, 0);
    assert_eq!(result.unwrap_err(), PlayerTurnError::TooManyCardsFlipped);
}
