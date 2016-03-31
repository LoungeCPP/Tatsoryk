(function() {
    'use strict';
    var Game = window.Game = window.Game || {};

    // TODO some vector library
    Game.makeVector = function(x, y) {
        return { x: x, y: y };
    };

    Game.normVector = function(vector) {
        var magnitude = Math.sqrt(vector.x * vector.x + vector.y * vector.y);
        if (magnitude != 0) {
            return Game.makeVector(vector.x / magnitude, vector.y / magnitude);
        } else {
            return vector;
        }
    };
})();
