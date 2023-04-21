use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use strato::{
    self,
    card::Deck,
    game::{GameEvent, GameStartupError, GameState, PlayerTurnError, StratoGame},
    player::{EndAction, StartAction},
};

fn start_game() -> (StratoGame<'static>, String, String) {
    let mut game = StratoGame::new();
    let player_1_id = game.add_player("Parker").unwrap();
    let player_2_id = game.add_player("Trevor").unwrap();
    game.start().unwrap();
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
    game.add_player("Parker").unwrap();
    assert_eq!(game.state, GameState::WaitingForPlayers);
}

#[test]
fn a_game_can_be_started() {
    let mut game = StratoGame::new();
    game.add_player("Parker").unwrap();
    game.add_player("Trevor").unwrap();
    let result = game.start();
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
    let (game, player_1_id, _) = start_game();
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
    let (mut game, _, _) = start_game();
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
    let player_1_id = game.add_player("Parker").unwrap();
    let player_2_id = game.add_player("Lexi").unwrap();
    let player_1 = game.get_player(player_1_id).unwrap();
    let player_2 = game.get_player(player_2_id).unwrap();
    assert!(game.list_players().iter().eq(vec![player_1, player_2]));
}

#[test]
fn cant_change_players_after_game_starts() {
    let mut game = StratoGame::new();
    game.add_player("Parker").unwrap();
    game.add_player("Lexi").unwrap();
    game.start().unwrap();
    assert_eq!(game.state, GameState::Active);

    let player_3_id = game.add_player("Trevor");
    assert_eq!(game.list_players().len(), 2);
    assert!(player_3_id.is_err());
}

#[test]
fn the_first_turn_can_take_from_discard_pile() {
    let (mut game, player_1_id, _) = start_game();
    let turn = game.start_player_turn(&player_1_id, StartAction::TakeFromDiscardPile);
    assert!(turn.is_ok());
}

#[test]
fn a_player_can_draw_and_flip() {
    let (mut game, player_1_id, _) = start_game();
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
    let (mut game, player_1_id, _) = start_game();
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
    let (mut game, player_1_id, player_2_id) = start_game();

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
    let (mut game, player_1_id, _) = start_game();

    game.start_player_turn(&player_1_id, StartAction::DrawFromDeck)
        .expect("Couldn't start Player 1's turn");
    let result = game.start_player_turn(&player_1_id, StartAction::DrawFromDeck);

    assert_eq!(result.unwrap_err(), PlayerTurnError::TurnAlreadyStarted);
}

#[test]
fn multiple_players_session_1() {
    let mut game = StratoGame::new();
    let cassie_id = game.add_player("Cassie").unwrap();
    let james_id = game.add_player("James").unwrap();
    game.start().unwrap();

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
fn can_subscribe_to_state_changes() {
    let mut game = StratoGame::new();
    let callback_triggered = Arc::new(AtomicBool::new(false));

    game.subscribe({
        let callback_triggered = callback_triggered.clone();

        move |e| {
            callback_triggered.store(true, Ordering::Relaxed);

            assert!(
                e == GameEvent::StateChange(&GameState::Startup)
                    || e == GameEvent::StateChange(&GameState::Active)
            );
        }
    });

    let player_1_id = game.add_player("Parker").unwrap();
    let _ = game.add_player("Trevor").unwrap();
    game.start().unwrap();

    game.start_player_turn(&player_1_id, StartAction::DrawFromDeck)
        .expect("Couldn't start turn");

    assert_eq!(callback_triggered.load(Ordering::Relaxed), true);
}
