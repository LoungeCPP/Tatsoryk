import {GameInput, MousePos} from './input';
import {GameSocket, Entity, MessageData} from './protocol';

export class GameScreen {
    public state: MessageData.WorldState;
    public bullet_ownership: { [bullet_id: number]: number } = {};
    playerId: number;

    // Current mouse position relative to canvas origin.
    // For aim vector rendering.
    mousePos: MousePos = null;

    keymap: { [type: number]: boolean } = {};
    socket: GameSocket;

    constructor(playerId: number, public playerSize: number, public bulletSize: number, startingState: MessageData.WorldState, socket: GameSocket) {
        this.playerId = playerId;
        this.state = startingState;
        this.socket = socket;
    }

    // Get the current player entity or null if you are dead.
    getPlayer(): Entity {
        var temp = this.state.alivePlayers.filter((player: Entity): boolean => {
            return player.id === this.playerId;
        });

        return temp[0] || null;
    }

    // Update your movement vector in correspondance to the currently pressed keys.
    updateMoving(): void {
        var move_x = 0;
        var move_y = 0;

        if (this.keymap[GameInput.MOVE_UP]) {
            move_y += -1;
        }
        if (this.keymap[GameInput.MOVE_DOWN]) {
            move_y += 1;
        }
        if (this.keymap[GameInput.MOVE_LEFT]) {
            move_x += -1;
        }
        if (this.keymap[GameInput.MOVE_RIGHT]) {
            move_x += 1;
        }

        if (move_x === 0 && move_y === 0) {
            this.socket.stopMoving();
        } else {
            this.socket.startMoving(new Victor(move_x, move_y));
        }
    }

    // Process a mouse click
    onMouseClick(x: number, y: number, type: GameInput): void {
        var player = this.getPlayer();
        if (player == null) {
            return;
        }

        var dx = x - player.position.x;
        var dy = y - player.position.y;

        this.socket.fire(new Victor(dx, dy));
    }

    // Process a mouse move
    onMouseMove(x: number, y: number): void {
        this.mousePos = {
            x: x,
            y: y,
        };
    }

    // Return a key down event processor bound to specified GameInput
    onKeyDown(key: GameInput): () => void {
        return (): void => {
            this.keymap[key] = true;
            this.updateMoving();
        };
    }

    // Return a key up event processor bound to specified GameInput
    onKeyUp(key: GameInput): () => void {
        return (): void => {
            this.keymap[key] = false;
            this.updateMoving();
        };
    }

    // Draw the current game.
    draw(context: CanvasRenderingContext2D): void {
        if (this.mousePos) {
            var curPlayer = this.getPlayer();
            if (curPlayer) {
                context.strokeStyle = 'rgba(255, 0, 0, 0.1)';
                context.beginPath();
                context.moveTo(curPlayer.position.x, curPlayer.position.y);
                context.lineTo(this.mousePos.x, this.mousePos.y);
                context.stroke();
                context.strokeStyle = 'black';
            }
        }

        for (var i = 0; i < this.state.alivePlayers.length; i++) {
            var player = this.state.alivePlayers[i];

            if (player.id === this.playerId) {
                context.fillStyle = 'red';
            } else {
                context.fillStyle = 'black';
            }

            context.beginPath();
            context.arc(player.position.x, player.position.y, this.playerSize, 0, 2 * Math.PI, true);
            context.fill();
        }

        for (var i = 0; i < this.state.aliveBullets.length; i++) {
            var bullet = this.state.aliveBullets[i];

            context.beginPath();
            context.arc(bullet.position.x, bullet.position.y, this.bulletSize, 0, 2 * Math.PI, true);
            context.stroke();
        }
    }
}
