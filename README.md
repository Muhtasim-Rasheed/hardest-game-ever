# Hardest Game Ever

## Description

This is a game where you are a wave and there are obstacles in your way.  
They can be both moving and static. You have to avoid them and reach the end of the level.  
(Spoiler: there isn't an end yet)  
There is a leaderboard!

## Wait! Check if you have these:

1. A computer
2. A keyboard (optional)
3. A mouse
4. A screen
6. [[HARDEST REQUIREMENT]] Internet connection

## How to run

### Client

1. Get it from here: `git clone https://github.com/Muhtasim-Rasheed/hardest-game-ever.git`
2. Go to the directory: `cd hardest-game-ever`
3. `cargo run --bin client`

### Server
You may only want to run the server if you want to host your own leaderboard, if so,
you need to change the code where the client gets the leaderboard from and submits to.  
[[UPDATE]] Now you can run the program in debug mode to connect to http://127.0.0.1:3000.

1. Get it from here: `git clone https://github.com/Muhtasim-Rasheed/hardest-game-ever.git`
2. Go to the directory: `cd hardest-game-ever`
3. `cargo run --bin server`

## How to play

- space or click: jump
- esc: exit out of screens
