(function() {
    'use strict';

    var Game = window.Game = window.Game || {};

    //
    // The main state holding class for the game.
    // It deals with input and rendering.
    //

    // Create a game screen.
    // The startingState must be a world state message.
    // The socket must be of type Game.Socket.
    Game.GameScreen = function(playerId, playerSize, bulletSize, startingState, socket) {
        var state = startingState;
        var keymap = {};
        var self = this;

        // Current mouse position relative to canvas origin.
        // For aim vector rendering.
        var mousePos = null;

        // Get the current player entity or null if you are dead.
        var getPlayer = function() {
            var temp = state.alivePlayers.filter(function(player) {
                return player.id === playerId;
            });

            return temp[0] || null;
        }

        // Update your movement vector in correspondance to the currently pressed keys.
        var updateMoving = function(){
            var move_x = 0;
            var move_y = 0;

            if (keymap[Game.INPUT_MOVE_UP]) {
                move_y += -1;
            }
            if (keymap[Game.INPUT_MOVE_DOWN]) {
                move_y += 1;
            }
            if (keymap[Game.INPUT_MOVE_LEFT]) {
                move_x += -1;
            }
            if (keymap[Game.INPUT_MOVE_RIGHT]) {
                move_x += 1;
            }

            if (move_x === 0 && move_y === 0) {
                socket.stopMoving();
            } else {
                socket.startMoving(new Victor(move_x, move_y));
            }
        };

        // Process a mouse click
        self.onMouseClick = function(x, y, type) {
            var player = getPlayer();
            if (player == null) {
                return;
            }

            var dx = x - player.position.x;
            var dy = y - player.position.y;

            socket.fire(new Victor(dx, dy));
        }

        // Process a mouse move
        self.onMouseMove = function(x, y) {
            mousePos = {
                x: x,
                y: y,
            };
        }

        // Process a key down event
        self.onKeyDown = function(key) {
            keymap[key] = true;
            updateMoving();
        };

        // Process a key up event
        self.onKeyUp = function(key) {
            keymap[key] = false;
            updateMoving();
        };

        // Draw the current game.
        self.draw = function(context) {
            if (mousePos) {
                var curPlayer = getPlayer();
                if (curPlayer) {
                    context.strokeStyle = 'rgba(255, 0, 0, 0.1)';
                    context.beginPath();
                    context.moveTo(curPlayer.position.x, curPlayer.position.y);
                    context.lineTo(mousePos.x, mousePos.y);
                    context.stroke();
                    context.strokeStyle = 'black';
                }
            }

            for (var i = 0; i < state.alivePlayers.length; i++) {
                var player = state.alivePlayers[i];

                if (player.id === playerId) {
                    context.fillStyle = 'red';
                } else {
                    context.fillStyle = 'black';
                }

                context.beginPath();
                context.arc(player.position.x, player.position.y, playerSize, 0, 2 * Math.PI, true);
                context.fill();
            }

            for (var i = 0; i < state.aliveBullets.length; i++) {
                var bullet = state.aliveBullets[i];

                context.beginPath();
                context.arc(bullet.position.x, bullet.position.y, bulletSize, 0, 2 * Math.PI, true);
                context.stroke();
            }
        };

        // Update your state
        self.updateState = function(newState) {
            state = newState;
        };
    };
})();
