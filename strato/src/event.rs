use anyhow::Result;

use crate::{
    game::{GameContext, GameOptions, GameStartupError, GameState},
    player::Player,
};

pub type ActionResult = Result<(Option<GameState>, Option<GameContext>)>;

pub trait Action {
    fn execute(&self, context: GameContext, state: GameState) -> ActionResult;
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameEvent<'a> {
    GameStart(GameStartAction),
    GameStartWithOptions(GameStartWithOptionsAction),
    RegisterPlayer(RegisterPlayerAction<'a>),
}

/**
 * Implementation
 */

#[derive(Debug, Clone, PartialEq)]
pub struct GameStartAction;

impl Action for GameStartAction {
    fn execute(&self, context: GameContext, state: GameState) -> ActionResult {
        handle_start(context, state, GameOptions::default())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameStartWithOptionsAction(pub GameOptions);

impl Action for GameStartWithOptionsAction {
    fn execute(&self, context: GameContext, state: GameState) -> ActionResult {
        handle_start(context, state, self.0.clone())
    }
}

fn handle_start(context: GameContext, state: GameState, options: GameOptions) -> ActionResult {
    let mut context = context;
    let mut state = state;

    context.deck.shuffle();
    let top_card = context.deck.draw().unwrap();
    context.discard_pile.put(top_card);
    // TODO: shuffle player order?
    deal_cards_to_players(&mut context)?;

    if let Some(first_player_idx) = options.first_player_idx {
        context.current_player_idx = Some(first_player_idx);
        state = GameState::Active;
    } else {
        state = GameState::DetermineFirstPlayer;
    }

    return Ok((Some(state), Some(context)));
}

fn deal_cards_to_players(context: &mut GameContext) -> Result<(), GameStartupError> {
    for player in context.players.iter_mut() {
        for row in 0..3 {
            for column in 0..4 {
                let card = context.deck.draw().ok_or(GameStartupError::DeckEmpty)?;
                player.spread.place_at(card, row, column)?;
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegisterPlayerAction<'a>(pub &'a str);

impl Action for RegisterPlayerAction<'_> {
    fn execute(&self, context: GameContext, _: GameState) -> ActionResult {
        // TODO: I don't like that I have to clone this here
        let mut context = context.clone();

        let player = Player::new(&self.0);
        context.players.push(player);

        Ok((None, Some(context)))
    }
}
