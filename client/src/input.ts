export const enum GameInput {
    MOVE_UP,
    MOVE_DOWN,
    MOVE_LEFT,
    MOVE_RIGHT,
    FIRE,
}

export type MousePos = {
    x: number;
    y: number;
};

// Get the mouse coords relative to the element
export function getRelativeMouseCords(event: MouseEvent, relativeTo: Element): MousePos {
    var rect = relativeTo.getBoundingClientRect();
    var borderWidth = 1;
    return {
        x: event.clientX - rect.left - borderWidth,
        y: event.clientY - rect.top - borderWidth,
    }
}
