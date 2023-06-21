use strato::{
    self,
    game::{AddPlayerAction, GameEvent::*, GameState, StratoGame},
    player::{generate_player_id, EndAction, StartAction},
};

// #[test]
fn full_game_1() {
    let mut game = StratoGame::new();
    let jackie_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &jackie_id }));
    let bryan_id = generate_player_id();
    game.send(AddPlayer(AddPlayerAction { id: &bryan_id }));
    game.start().unwrap();

    /*
     * Determine first player
     */

    assert_eq!(game.state, GameState::DetermineFirstPlayer);

    // TODO: figure out who goes first

    assert_eq!(game.state, GameState::Active);

    /*
     * Round #1
     */

    game.start_player_turn(&jackie_id, StartAction::DrawFromDeck)
        .expect("Couldn't start jackie's turn");
    game.end_player_turn(&jackie_id, EndAction::Flip { row: 1, column: 2 })
        .expect("Couldn't end jackie's turn");
    assert_eq!(game.context.discard_pile.size(), 2);

    game.start_player_turn(&bryan_id, StartAction::TakeFromDiscardPile)
        .expect("Couldn't start bryan's turn");
    game.end_player_turn(&bryan_id, EndAction::Swap { row: 2, column: 2 })
        .expect("Couldn't end bryan's turn");
    assert_eq!(game.context.discard_pile.size(), 2); // hasn't changed because this was taken from discard pile

    /*
     * Round #2
     */

    game.start_player_turn(&jackie_id, StartAction::DrawFromDeck)
        .expect("Couldn't start jackie's 2nd turn");
    game.end_player_turn(&jackie_id, EndAction::Swap { row: 2, column: 3 })
        .expect("Couldn't end jackie's 2nd turn");
    assert_eq!(game.context.discard_pile.size(), 3);

    game.start_player_turn(&bryan_id, StartAction::DrawFromDeck)
        .expect("Couldn't start bryan's 2nd turn");
    game.end_player_turn(&bryan_id, EndAction::Flip { row: 0, column: 0 })
        .expect("Couldn't end bryan's 2nd turn");
    assert_eq!(game.context.discard_pile.size(), 4);

    // TODO: test how many are flipped over by now
}
