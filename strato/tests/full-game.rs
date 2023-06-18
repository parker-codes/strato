use strato::{
    self,
    game::{GameOptions, GameState, StratoGame},
    player::{EndAction, StartAction},
};

// #[test]
fn full_game_1() {
    let mut game = StratoGame::new();
    let jackie_id = game.add_player("Jackie").unwrap();
    let bryan_id = game.add_player("Bryan").unwrap();
    game.start().unwrap();

    /*
     * Determine first player
     */

    assert!(game.state_matches(GameState::DetermineFirstPlayer));

    // TODO: figure out who goes first

    assert!(game.state_matches(GameState::Active));

    /*
     * Round #1
     */

    game.start_player_turn(&jackie_id, StartAction::DrawFromDeck)
        .expect("Couldn't start jackie's turn");
    game.end_player_turn(&jackie_id, EndAction::Flip { row: 1, column: 2 })
        .expect("Couldn't end jackie's turn");
    assert_eq!(game.context.lock().unwrap().discard_pile.size(), 2);

    game.start_player_turn(&bryan_id, StartAction::TakeFromDiscardPile)
        .expect("Couldn't start bryan's turn");
    game.end_player_turn(&bryan_id, EndAction::Swap { row: 2, column: 2 })
        .expect("Couldn't end bryan's turn");
    assert_eq!(game.context.lock().unwrap().discard_pile.size(), 2); // hasn't changed because this was taken from discard pile

    /*
     * Round #2
     */

    game.start_player_turn(&jackie_id, StartAction::DrawFromDeck)
        .expect("Couldn't start jackie's 2nd turn");
    game.end_player_turn(&jackie_id, EndAction::Swap { row: 2, column: 3 })
        .expect("Couldn't end jackie's 2nd turn");
    assert_eq!(game.context.lock().unwrap().discard_pile.size(), 3);

    game.start_player_turn(&bryan_id, StartAction::DrawFromDeck)
        .expect("Couldn't start bryan's 2nd turn");
    game.end_player_turn(&bryan_id, EndAction::Flip { row: 0, column: 0 })
        .expect("Couldn't end bryan's 2nd turn");
    assert_eq!(game.context.lock().unwrap().discard_pile.size(), 4);

    // TODO: test how many are flipped over by now
}
