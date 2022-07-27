use core::{
    self,
    card::{Card, Deck},
    game::{GameState, PlayerTurnError, StratoGame},
    player::{EndAction, StartAction},
};

#[test]
fn a_game_can_be_initialized() {
    let game = StratoGame::new();
    assert_eq!(game.state, GameState::WaitingForPlayers);
    assert_eq!(game.context.deck.size(), Deck::EMPTY_SIZE);
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
    game.start().unwrap();
    assert_eq!(game.state, GameState::Active);
}

#[test]
fn cant_start_without_players() {
    let mut game = StratoGame::new();
    game.start().unwrap();
    assert_eq!(game.state, GameState::WaitingForPlayers);
}

#[test]
fn a_started_game_deals_cards_to_players() {
    let mut game = StratoGame::new();
    let player_id = game.add_player("Joe").unwrap();
    game.start().unwrap();
    let player = game.get_player(player_id).unwrap();
    assert_eq!(
        player
            .spread
            .view()
            .into_iter()
            .flatten()
            .filter(|x| x.is_some() && !x.unwrap().is_flipped())
            .collect::<Vec<_>>()
            .len(),
        12
    );
    assert_eq!(game.context.deck.size(), Deck::FULL_SIZE - 12);
}

#[test]
fn starting_multiple_times_is_inconsequential() {
    let mut game = StratoGame::new();
    game.add_player("Parker").unwrap();
    game.start().unwrap();
    let deck_snapshot = game.context.deck.clone();
    assert!(game.start().is_err());
    assert!(game.start().is_err());
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
fn a_player_can_draw_and_flip() {
    let mut game = StratoGame::new();
    let player_id = game.add_player("Trevor").unwrap();
    game.start().unwrap();

    game.start_player_turn(&player_id, StartAction::DrawFromDeck)
        .expect("Couldn't start turn");
    assert!(game.get_player(&player_id).unwrap().holding().is_some());
    game.end_player_turn(&player_id, EndAction::Flip { row: 1, column: 2 })
        .expect("Couldn't end turn");
    assert!(game.get_player(&player_id).unwrap().holding().is_none());
    assert_eq!(game.context.discard_pile.size(), 1);
}

#[test]
fn the_first_turn_cant_take_from_discard_pile() {
    let mut game = StratoGame::new();
    let player_id = game.add_player("Kristen").unwrap();
    game.start().unwrap();

    let turn = game.start_player_turn(&player_id, StartAction::TakeFromDiscardPile);
    assert_eq!(turn.unwrap_err(), PlayerTurnError::DiscardPileEmpty);
}

#[test]
fn a_player_can_take_and_swap() {
    let mut game = StratoGame::new();
    let player_id = game.add_player("Jon").unwrap();
    game.start().unwrap();

    // have to add a card to the discard pile first!
    game.context.discard_pile.put(Card::new(-2));

    game.start_player_turn(&player_id, StartAction::TakeFromDiscardPile)
        .expect("Couldn't start turn");
    assert!(game.get_player(&player_id).unwrap().holding().is_some());
    game.end_player_turn(&player_id, EndAction::Swap { row: 2, column: 2 })
        .expect("Couldn't end turn");
    assert!(game.get_player(&player_id).unwrap().holding().is_none());
    assert_eq!(game.context.discard_pile.size(), 1);
}

#[test]
fn cant_flip_same_card_twice() {
    let mut game = StratoGame::new();
    let player_id = game.add_player("Julie").unwrap();
    game.start().unwrap();

    const ROW: usize = 0;
    const COLUMN: usize = 1;

    // First turn
    game.start_player_turn(&player_id, StartAction::DrawFromDeck)
        .expect("Couldn't start turn");
    game.end_player_turn(
        &player_id,
        EndAction::Flip {
            row: ROW,
            column: COLUMN,
        },
    )
    .expect("Couldn't end turn");

    // Second turn
    game.start_player_turn(&player_id, StartAction::DrawFromDeck)
        .expect("Couldn't start turn");
    let turn = game.end_player_turn(
        &player_id,
        EndAction::Flip {
            row: ROW,
            column: COLUMN,
        },
    );
    assert!(turn.is_err());
}

#[test]
fn multiple_players_session_1() {
    let mut game = StratoGame::new();
    let cassie_id = game.add_player("Cassie").unwrap();
    let james_id = game.add_player("James").unwrap();
    game.start().unwrap();

    // Cassie first
    game.start_player_turn(&cassie_id, StartAction::DrawFromDeck)
        .expect("Couldn't start turn");
    game.end_player_turn(&cassie_id, EndAction::Flip { row: 1, column: 2 })
        .expect("Couldn't end turn");
    assert_eq!(game.context.discard_pile.size(), 1);

    // James next
    game.start_player_turn(&james_id, StartAction::TakeFromDiscardPile)
        .expect("Couldn't start turn");
    game.end_player_turn(&james_id, EndAction::Swap { row: 2, column: 2 })
        .expect("Couldn't end turn");
    assert_eq!(game.context.discard_pile.size(), 1); // hasn't changed because this was taken from discard pile

    // Then Cassie again
    game.start_player_turn(&cassie_id, StartAction::DrawFromDeck)
        .expect("Couldn't start turn");
    game.end_player_turn(&cassie_id, EndAction::Swap { row: 2, column: 3 })
        .expect("Couldn't end turn");
    assert_eq!(game.context.discard_pile.size(), 2);

    // Then James again
    game.start_player_turn(&james_id, StartAction::DrawFromDeck)
        .expect("Couldn't start turn");
    game.end_player_turn(&james_id, EndAction::Flip { row: 0, column: 0 })
        .expect("Couldn't end turn");
    assert_eq!(game.context.discard_pile.size(), 3);
}
