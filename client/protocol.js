(function() {
    'use strict';
    var Game = window.Game = window.Game || {};

    //
    // Game protocol implementation
    //
    // Game.Socket uses Transport to exchange messages with the server, and then
    // dispatches them as events (with the ID being the same as in the spec).
    // TODO validation maybe; could use JSON Schema also
    //
    // Actual handling logic is in game.js.
    //

    Game.Socket = function(transport) {
        var self = this;
        var messages = {};

        messages.welcome = function(msg) {
            return {
                id: msg.id,
                speed: msg.speed,
                size: msg.size,
                bulletSpeed: msg.bullet_speed,
                bulletSize: msg.bullet_size,
            };
        };

        messages.go_away = function(msg) {
            transport.disconnect();
            return {
                reason: msg.reason,
            };
        };

        messages.player_joined = messages.player_left = function(msg) {
            return {
                id: msg.id
            };
        };

        messages.shots_fired = function(msg) {
            return {
                id: msg.id,
                bulletID: msg.bullet_id,
                position: Game.makeVector(msg.x, msg.y),
                aim: Game.makeVector(msg.aim_x, msg.aim_y),
            };
        };

        messages.player_spawned = function(msg) {
            return {
                id: msg.id,
                position: Game.makeVector(msg.x, msg.y),
            };
        };

        messages.player_destroyed = function(msg) {
            return {
                id: msg.id,
                killerID: msg.killer_id || null,
                bulletID: msg.bullet_id || null,
            };
        };

        messages.player_moving = function(msg) {
            return {
                id: msg.id,
                position: Game.makeVector(msg.x, msg.y),
                direction: Game.makeVector(msg.move_x, msg.move_y),
            };
        };

        messages.player_stopped = function(msg) {
            return {
                id: msg.id,
                position: Game.makeVector(msg.x, msg.y),
            };
        };

        messages.world_state = function(msg) {
            var entityMap = function(entity) {
                var result = {
                    id: entity.id,
                    position: Game.makeVector(entity.x, entity.y),
                };

                if (entity.move_x !== undefined && entity.move_y !== undefined) {
                    result.direction = Game.makeVector(entity.move_x, entity.move_y);
                }

                return result;
            };

            return {
                playerCount: msg.player_count,
                alivePlayers: msg.alive_players.map(entityMap),
                aliveBullets: msg.alive_bullets.map(entityMap),
            };
        };

        var dispatch = function(msg) {
            var handleMessage = messages[msg.type];

            if (handleMessage === undefined) {
                console.error('Socket: received invalid msg.type %s', msg.type);
                return;
            }

            var message = handleMessage(msg.data);
            console.log('Socket: dispatching event %s with data %o', msg.type, message);
            self.emitEvent(msg.type, [message]);
        };

        var send = function(type, data) {
            var message = { type: type, data: data };
            console.log('Socket: sending message %s with data %o', type, data);
            transport.send({ type: type, data: data });
        };

        self.startMoving = function(moveVector) {
            moveVector = Game.normVector(moveVector);
            send('start_moving', { move_x: moveVector.x, move_y: moveVector.y });
        };

        self.stopMoving = function() {
            send('stop_moving');
        };

        self.fire = function(aimVector) {
            aimVector = Game.normVector(aimVector);
            // TODO spec: rename to aim_x, aim_y
            send('fire', { move_x: aimVector.x, move_y: aimVector.y });
        };

        transport.addListener('message', dispatch);
    };

    heir.inherit(Game.Socket, EventEmitter);
})();
