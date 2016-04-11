import EventEmitter = require("wolfy87-eventemitter");
import {GameWSTransport} from './transport';

//
// Game protocol implementation
//
// GameSocket uses Transport to exchange messages with the server, and then
// dispatches them as events (with the ID being the same as in the spec).
// TODO validation maybe; could use JSON Schema also
//
// Actual handling logic is in game.js.
//

export class GameSocket extends EventEmitter {
    transport: GameWSTransport;
    message_decomposers: { [type: string]: (message: Object) => MessageData };

    constructor(transport: GameWSTransport) {
        super();
        this.transport = transport;
        this.message_decomposers = make_message_decomposers(transport);

        transport.addListener('message', this.dispatch)
    }

    dispatch = (msg: Message): void => {
        var handleMessage = this.message_decomposers[msg.type];

        if (handleMessage === undefined) {
            console.error('Socket: received invalid msg.type %s', msg.type);
            return;
        }

        var message = handleMessage(msg.data);
        this.emitEvent(msg.type, [message]);
    }

    send(type: MessageType, data: MessageData): void {
        var message = { type: type, data: data };
        this.transport.send({ type: type, data: data });
    }

    startMoving(moveVector: Victor): void {
        moveVector = moveVector.norm();
        this.send('start_moving', <MessageData.StartMoving>{
            move_x: moveVector.x,
            move_y: moveVector.y,
        });
    }

    stopMoving(): void {
        this.send('stop_moving', <MessageData.StopMoving>{});
    }

    fire(aimVector: Victor): void {
        aimVector = aimVector.norm();
        // TODO spec: rename to aim_x, aim_y
        this.send('fire', <MessageData.Fire>{
            move_x: aimVector.x,
            move_y: aimVector.y,
        });
    }
}

export class Entity {
    constructor(public id: number, public position: Victor, public direction?: Victor) { }

    public distanceSq(to: Entity | Victor): number {
        if ('position' in to) {
            return this.position.distanceSq((<Entity>to).position);
        } else {
            return this.position.distanceSq(<Victor>to);
        }
    }
}

export class Message {
    constructor(public type: MessageType, public data: MessageData) { }
}

interface MessageData { }

export namespace MessageData {
    export interface Welcome extends MessageData {
        id: number,
        speed: number,
        size: number,
        bulletSpeed: number,
        bulletSize: number,
    }

    export interface GoAway extends MessageData {
        reason: string,
    }

    export interface PlayerJoined extends MessageData {
        id: number,
    }

    export interface PlayerLeft extends MessageData {
        id: number,
    }

    export interface ShotsFired extends MessageData {
        id: number,
        bulletID: number,
        position: Victor,
        aim: Victor,
    }

    export interface PlayerSpawned extends MessageData {
        id: number,
        position: Victor,
    }

    export interface PlayerDestroyed extends MessageData {
        id: number,
        killerID: number,
        bulletID: number,
    }

    export interface PlayerMoving extends MessageData {
        id: number,
        position: Victor,
        direction: Victor,
    }

    export interface PlayerStopped extends MessageData {
        id: number,
        position: Victor,
    }

    export interface WorldState extends MessageData {
        playerCount: number,
        alivePlayers: Array<Entity>,
        aliveBullets: Array<Entity>,
    }

    export interface StartMoving extends MessageData {
        move_x: number,
        move_y: number,
    }

    export interface StopMoving extends MessageData { }

    export interface Fire extends MessageData {
        move_x: number,
        move_y: number,
    }
}

// It's a string enum, no worries
// https://github.com/Microsoft/TypeScript/issues/3192#issuecomment-181363162
type MessageType =
    'welcome' |
    'go_away' |
    'player_joined' |
    'player_left' |
    'shots_fired' |
    'player_spawned' |
    'player_destroyed' |
    'player_moving' |
    'player_stopped' |
    'world_state' |
    'start_moving' |
    'stop_moving' |
    'fire';
const MessageType = {
    Welcome: 'welcome' as MessageType,
    GoAway: 'go_away' as MessageType,
    PlayerJoined: 'player_joined' as MessageType,
    PlayerLeft: 'player_left' as MessageType,
    ShotsFired: 'shots_fired' as MessageType,
    PlayerSpawned: 'player_spawned' as MessageType,
    PlayerDestroyed: 'player_destroyed' as MessageType,
    PlayerMoving: 'player_moving' as MessageType,
    PlayerStopped: 'player_stopped' as MessageType,
    WorldState: 'world_state' as MessageType,
    StartMoving: 'start_moving' as MessageType,
    StopMoving: 'stop_moving' as MessageType,
    Fire: 'fire' as MessageType,
}

function make_message_decomposers(transport: GameWSTransport): { [type: string]: (message: Object) => MessageData } {
    return {
        'welcome': (msg: Object): MessageData.Welcome => {
            return {
                id: <number>(<any>msg)['id'],
                speed: <number>(<any>msg)['speed'],
                size: <number>(<any>msg)['size'],
                bulletSpeed: <number>(<any>msg)['bullet_speed'],
                bulletSize: <number>(<any>msg)['bullet_size'],
            };
        },
        'go_away': (msg: Object): MessageData.GoAway => {
            transport.disconnect();
            return {
                reason: <string>(<any>msg)['reason'],
            };
        },
        'player_joined': (msg: Object): MessageData.PlayerJoined => {
            return {
                id: <number>(<any>msg)['id'],
            };
        },
        'player_left': (msg: Object): MessageData.PlayerLeft => {
            return {
                id: <number>(<any>msg)['id'],
            };
        },
        'shots_fired': (msg: Object): MessageData.ShotsFired => {
            return {
                id: <number>(<any>msg)['id'],
                bulletID: <number>(<any>msg)['bullet_id'],
                position: new Victor(<number>(<any>msg)['x'], <number>(<any>msg)['y']),
                aim: new Victor(<number>(<any>msg)['aim_x'], <number>(<any>msg)['aim_y']),
            };
        },
        'player_spawned': (msg: Object): MessageData.PlayerSpawned => {
            return {
                id: <number>(<any>msg)['id'],
                position: new Victor(<number>(<any>msg)['x'], <number>(<any>msg)['y']),
            };
        },
        'player_destroyed': (msg: Object): MessageData.PlayerDestroyed => {
            return {
                id: <number>(<any>msg)['id'],
                killerID: <number>(<any>msg)['killer_id'] || null,
                bulletID: <number>(<any>msg)['bullet_id'] || null,
            };
        },
        'player_moving': (msg: Object): MessageData.PlayerMoving => {
            return {
                id: <number>(<any>msg)['id'],
                position: new Victor(<number>(<any>msg)['x'], <number>(<any>msg)['y']),
                direction: new Victor(<number>(<any>msg)['move_x'], <number>(<any>msg)['move_y']),
            };
        },
        'player_stopped': (msg: Object): MessageData.PlayerStopped => {
            return {
                id: <number>(<any>msg)['id'],
                position: new Victor(<number>(<any>msg)['x'], <number>(<any>msg)['y']),
            };
        },
        'world_state': (msg: Object): MessageData.WorldState => {
            var entityMap = (entity: Object): Entity => {
                var entityId: number = (<any>entity)['id'];
                var entityPos = new Victor(<number>(<any>entity)['x'], <number>(<any>entity)['y']);
                var entityMoveX: number = (<any>entity)['move_x'];
                var entityMoveY: number = (<any>entity)['move_y'];
                if (entityMoveX !== undefined && entityMoveY !== undefined) {
                    var entityMove = new Victor(entityMoveX, entityMoveY);
                    return new Entity(entityId, entityPos, entityMove);
                } else {
                    return new Entity(entityId, entityPos);
                }
            };

            return {
                playerCount: <number>(<any>msg)['player_count'],
                alivePlayers: (<Array<Object>>(<any>msg)['alive_players']).map(entityMap),
                aliveBullets: (<Array<Object>>(<any>msg)['alive_bullets']).map(entityMap),
            };
        },
    };
}
