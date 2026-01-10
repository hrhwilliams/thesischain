import { DeviceContext } from '../../pkg/end2_wasm_client'

export type InboundChatMessage = {
    message_id: string,
    device_id: string,
    ciphertext: string,
    timestamp: string,
    is_pre_key: boolean,
}

export type ChatMessage = {
    id: string,
    author: string,
    author_id: string,
    content: string,
    timestamp: string,
}

export type DeviceContextKeys = {
    x25519: string,
    ed25519: string,
    signature: string,
}

export type DeviceOneTimeKeys = {
    keys: [string],
    signature: string,
}

let instance: End2DeviceContext | null = null

class End2DeviceContext {
    device: DeviceContext

    constructor(user_id: string, device_id: string) {
        const savedState = localStorage.getItem(`end2_device_identity_${device_id}`)

        if (savedState) {
            this.device = DeviceContext.try_from_state(savedState)
        } else {
            this.device = DeviceContext.new(user_id, device_id)
        }
    }

    save_state() {
        localStorage.setItem(`end2_device_identity_${this.device.device_id()}`, this.device.export_state())
    }

    encrypt(channel_id: string, plaintext: string) {
        return this.device.encrypt(channel_id, plaintext)
    }

    decrypt(channel_id: string, message: InboundChatMessage) {
        return this.device.decrypt(channel_id, message)
    }

    get_keys(): DeviceContextKeys {
        return this.device.get_identity_keys()
    }

    generate_otks(count: number): DeviceOneTimeKeys {
        return this.device.generate_otks(count)
    }
}
