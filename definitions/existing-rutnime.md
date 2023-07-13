# Existing Runtime

What it needs:

GameState {
  inventory: Vec<Item>,
  variables: Vec<Resource>,

}

0) set_seed(seed: int)

1) get_event(state: GameState) -> Option<Event>
  1) Por cada requerimiento cumplido aumenta el peso
  2) Aplico el cooldown a los eventos que salieron si están debajo de un threshold
  3) Random para seleccionar evento
  4) Devovler evento con opciones disponibles validas y bajar el peso
  6) Reset de modificadores seteados en (1)

2) choose(n) -> Option<Result>
  1) dada la opcion elegida, compruebo que la puede elegir
  2) get random result
  3) apply effects
  4) set important decision if needed
  5) return effect result

