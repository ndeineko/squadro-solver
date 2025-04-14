# Squadro-solver

***Squadro-solver*** is a program designed to fully explore and solve the strategy board game *Squadro*[^1].

For every reachable game state, it determines which player (if any) has a guaranteed path to victory.

## Main findings

*Note : by convention, the board is rotated so that one player starts from the left while the other starts from the top.*

- The *left* player can a guarantee a victory from the starting position, even when playing second.
- There are **46199129613** reachable game states in total.
- Assuming *perfect play* from both players :
  - **21681412181** (≈ 46.93%) of these states are winning for the *top* player.
  - **24492844613** (≈ 53.02%) are winning for the *left* player.
  - **24872819** (≈ 0.05%) are drawing (the game will go on indefinitely).

## Work in progress

Source code and more details coming soon™.

In the meantime, you can play against *squadro-solver* [here](https://squadro-solver.netlify.app/). You cannot win against  the *left* player, but you can (relatively) easily win against the *top* player since the computer logic is simple :
- It plays any winning move when there is at least one.
- Otherwise and if there is a path to an infinite game, it plays a drawing move.
- Finally, if all moves are losing, it plays randomly.


## License

This project is licensed under the [MIT License](LICENSE).

[^1]: Squadro is a 2-player board game created by Adrian Jimenez Pascual (https://dirdam.github.io/squadro.html) and published by GIGAMIC (https://en.gigamic.com/modern-classics/504-squadro.html).
