# grim-patron-bot

Discord bot for tinkering with rust.


### Game
Copy of Mudae "blacktea" game:
1. Game is created, configured and started.
2. Each turn someone who joined the game receives a string of a length 3 and has to respond with a word containing it as a substring in a limited time.
3. Each time someone failes to respond with a fitting word, they lose 1 life. Once a player is out of lives they stop participating in a current game.
4. Turns continue until only 1 player left.

TODO (who needs issue tracker anyway):
1. Handle user registration: receive registrations, store them in a Game object.
1. Personalize rounds: tag users when sending a template, handle lives and user input.
1. Fix race conditions and make sure all jobs are killed once another game is started.
1. (optional) Add ability to run multiple games in multiple channels.
