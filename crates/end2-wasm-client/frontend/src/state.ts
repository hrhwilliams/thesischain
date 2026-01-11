import { defineStore } from 'pinia'
import { get_keys, get_my_otks, get_otks_for_channel, new_device_id, upload_keys, upload_otks, type UserInfo } from './api';
import { DeviceContext } from '../../pkg/end2_wasm_client';
import type { Channel, ChannelInfo, MessageReceivedReply, NewNickname } from './api'
import type { ChatMessage, DeviceContextKeys, InboundChatMessage } from './device'
import { toRaw } from 'vue';

type WsEvent =
    | { type: 'channel_created', data: Channel }
    | { type: 'message', data: InboundChatMessage }
    | { type: 'message_received', data: MessageReceivedReply }
    | { type: 'nickname_changed', data: NewNickname }

export const useClientState = defineStore('session', {
    state: () => ({
        user: null as UserInfo | null,
        known_users: {} as Record<string, UserInfo>,
        current_channel: null as string | null,
        channels: new Map<string, Channel>(),
        context: null as DeviceContext | null,
        channel_info: {} as Record<string, ChannelInfo>,
        messages: {} as Record<string, ChatMessage[]>,
        ws: null as WebSocket | null,
    }),

    getters: {
        channel_list: (state) => Array.from(state.channels.values()),
        is_logged_in: (state) => state.user !== null,
        this_device: (state) => state.context?.device_id(),
        crypto: (state) => state.context,
        socket: (state) => state.ws
    },

    actions: {
        login(user: UserInfo) {
            this.user = structuredClone(toRaw(user))
        },

        logout() {
            this.$reset()
        },

        init_ws() {
            if (!this.context) {
                console.error("unable to init websocket: no valid device")
                throw "no valid device"
            }

            this.ws = new WebSocket(`http://localhost:8081/api/me/device/${this.context.device_id()}/ws`)
            console.log("initializing ws")
            this.ws.onmessage = (event: MessageEvent) => {
                console.log(event.data)
                const payload = JSON.parse(event.data) as WsEvent

                try {
                    switch (payload.type) {
                        case 'channel_created':
                            this.add_channel(payload.data)
                            break
                        case 'message':
                            console.info('message', payload.data)
                            let message: ChatMessage
                            if (payload.data.is_pre_key) {
                                message = this.context?.decrypt_new_session(payload.data)
                            } else {
                                message = this.context?.decrypt(payload.data)
                            }

                            this.on_message_decrypted(payload.data.channel_id, message)
                            break
                        case 'message_received':
                            const received_message = this.context?.message_received(payload.data)
                            this.on_message_decrypted(payload.data.channel_id, received_message)
                            break
                        case 'nickname_changed':
                            if (this.user && payload.data.user_id === this.user.id) {
                                this.set_nickname(payload.data.nickname)
                            }
                            break
                        default:
                            console.warn('unknown ws event', payload)
                    }
                } catch (e) {
                    throw e
                } finally {
                    if (this.user && this.context) {
                        console.info('saving state')
                        localStorage.setItem(`end2_device_context_${this.user.id}`, this.context.export_state())
                    }
                }
            }
        },

        on_message_decrypted(channel_id: string, message: ChatMessage) {
            if (!this.messages[channel_id]) {
                this.messages[channel_id] = []
            }

            this.messages[channel_id].push(message)
        },

        get_messages(channel_id: string) {
            if (this.context) {
                try {
                    this.messages[channel_id] = this.context.get_message_history(channel_id)
                } catch (e) {}
            }
        },

        set_nickname(nickname: string) {
            if (this.user) {
                this.user.nickname = nickname
            }
        },

        add_channel(channel: Channel) {
            this.channels.set(channel.channel_id, channel)
        },

        get_user(channel_id: string, user_id: string) {
            if (this.user && user_id === this.user?.id) {
                return this.user.nickname ? this.user.nickname : this.user.username
            }

            const other_user = this.channels.get(channel_id)
            if (other_user && other_user.user_id === user_id) {
                return other_user.nickname ? other_user.nickname : other_user.username
            }

            return null
        },

        async init_device() {
            if (!this.user) {
                console.error("unable to init device: no valid user")
                throw "no valid user"
            }

            if (!this.context) {
                const saved_state = localStorage.getItem(`end2_device_context_${this.user.id}`)
                if (saved_state) {
                    this.context = DeviceContext.try_from_state(saved_state)
                } else {
                    const response = await new_device_id()
                    if (!response.ok) {
                        console.error("failed to fetch device ID")
                        throw "failed to fetch device ID"
                    }
    
                    this.context = DeviceContext.new(this.user.id, response.data.id)
                }
            }

            const response2 = await get_keys(this.context.device_id())
            if (response2.ok && !response2.data.ed25519 && !response2.data.x25519) {
                const keys: DeviceContextKeys  = this.context.get_identity_keys()
                let key_response = await upload_keys(this.context.device_id(), keys)

                if (key_response.ok) {
                    localStorage.setItem(`end2_device_context_${this.user.id}`, this.context.export_state())
                } else {
                    console.error("failed to upload keys")
                    throw "failed to upload keys"
                }
            }
        },

        async upload_otks() {
            if (!this.context) {
                console.error("failed to upload otks because context is not initialized")
                throw "failed to upload otks"
            }

            const otk_count = await get_my_otks(this.context.device_id())
            if (!otk_count.ok) {
                console.error("failed to fetch otks: ", otk_count.error.message)
                throw "failed to fetch otks"
            }

            if (otk_count.data.otks.length < 10) {
                const keys = this.context.generate_otks(50)
                const otk_upload = await upload_otks(this.context.device_id(), keys)

                if (!otk_upload.ok) {
                    console.error("failed to upload otks: ", otk_upload.error.message)
                    throw "failed to upload otks"
                }
            }
        },

        async create_sessions(channel_id: string, device_ids: string[]) {
            const otks = await get_otks_for_channel(channel_id, device_ids)

            otks.forEach(result => {
                if (!result.ok) {
                    throw result.error
                }

                if (!this.context) {
                    throw "failed to fetch otks"
                }

                this.context.create_session_from_otk(channel_id, result.data)
            })
        },

        async encrypt_and_send(channel_id: string, message: string) {
            if (!this.context || !this.ws) {
                console.error("device or ws not initialized")
                throw "device or ws not initialized"
            }

            const encrypted_message = this.context.encrypt(channel_id, message)
            this.ws.send(JSON.stringify(encrypted_message))
        },

        has_channel_info_for(channel_id: string): boolean {
            if (!this.context) {
                console.error("device not initialized")
                throw "device not initialized"
            }

            return this.context.has_channel_info_for(channel_id)
        },

        register_channel(channel_info: ChannelInfo) {
            if (!this.context) {
                console.error("failed to init channel because context is not initialized")
                throw "device not initialized"
            }

            try {
                this.current_channel = channel_info.channel_id
                this.context.initialize_for_channel(channel_info)
                this.channel_info[channel_info.channel_id] = channel_info
            } catch (e) {
                console.error("error initializing channel: ", e)
            }
        },

        devices_missing_sessions(channel_id: string): string[] {
            if (!this.context) {
                console.error("failed to init channel because context is not initialized")
                throw "device not initialized"
            }

            return this.context.missing_otks(channel_id)
        }
    }
})
