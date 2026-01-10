import { useClientState } from "./state"

const WS_URL = import.meta.env.VITE_WS_URL
let socket: WebSocket | null = null

export function connect_websocket() {
    if (socket) {
        return socket
    } else {
        socket = new WebSocket(WS_URL)
    }

    socket.onopen = () => {
        useClientState().ws_connected = true
    }

    socket.onmessage = (event: MessageEvent) => {
        console.log('websocket message', event.data)
    }

    socket.onclose = () => {
        useClientState().ws_connected = false
        socket = null
    }

    return socket
}

export function send_websocket(data: unknown): void {
    if (socket?.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify(data))
    }
}

export function disconnect_websocket(): void {
    socket?.close()
}
