// import { defineStore } from 'pinia'
// import { get_my_otks, get_otk, request } from './api';
// import { DeviceContext } from '../../pkg/end2_wasm_client';
// import type { ApiResult, MessageReceivedReply, NewNickname } from './api'
// import type { DeviceContextKeys, InboundChatMessage } from './crypto'
// import { get_devices, get_our_device, type DeviceInfo } from './device'
// import { toRaw } from 'vue';
// import type { ChannelInfo } from './channel';
// import type { UserInfo } from './user';

// type UserId = string
// type ChannelId = string
// type DeviceId = string

// /*
// current_user: null as UserId | null,              // how we identify ourselves
// users:    {} as Record<UserId, UserInfo>,         // info about all users we interact with
// channels: {} as Record<ChannelId, ChannelInfo>,   // info about all channels we are in
// messages: {} as Record<ChannelId, ChatMessage[]>, // all messages in a channel
// devices:  {} as Record<UserId, DeviceInfo[]>,     // info about all devices a user has
// context:  null as DeviceContext | null,           // crypto context
// ws:       null as WebSocket | null,
//  */

// type WsEvent =
//     | { type: 'channel_created', data: ChannelInfo }
//     | { type: 'message', data: InboundChatMessage }
//     | { type: 'message_received', data: MessageReceivedReply }
//     | { type: 'nickname_changed', data: NewNickname }

// export const useClientState = defineStore('session', {
//     state: () => ({
//         current_user: null as UserId | null,
//         users: {} as Record<UserId, UserInfo>,
//         channels: {} as Record<ChannelId, ChannelInfo>,
//         messages: {} as Record<ChannelId, ChatMessage[]>,
//         context: null as DeviceContext | null,
//         ws: null as WebSocket | null,
//     }),

//     getters: {
//         me: (state): UserInfo | null => {
//             if (state.current_user) {
//                 return state.users[state.current_user] ?? null
//             }

//             return null
//         },
//         is_logged_in: (state) => state.current_user !== null,
//         this_device: (state) => state.context?.device_id(),
//         crypto: (state) => state.context,
//         socket: (state) => state.ws
//     },

//     actions: {
//         login(user: UserInfo) {
//             this.users[user.id] = structuredClone(toRaw(user))
//             this.current_user = user.id
//         },

//         logout() {
//             this.$reset()
//         },

//         insert_user(user_id: string, user: UserInfo) {
//             this.users[user_id] = structuredClone(toRaw(user))
//         },

//         async hydrate_channel(channel: ChannelInfo) {
//             if (!this.context) {
//                 throw new Error("crypto context not initialized")
//             }

//             // 1. Insert users (UI concern)
//             for (const user of channel.participants) {
//                 if (!this.users[user.id]) {
//                     this.users[user.id] = structuredClone(toRaw(user))
//                 }
//             }

//             // 2. Insert channel (routing / UI concern)
//             this.channels[channel.channel_id] = structuredClone(toRaw(channel))


//             // 4. Initialize channel in crypto
//             this.context.initialize_channel(channel)
//         },

//         init_ws() {
//             if (!this.context) {
//                 console.error('unable to init websocket: no valid device')
//                 throw 'no valid device'
//             }

//             this.ws = new WebSocket(`http://localhost:8081/api/me/device/${this.context.device_id()}/ws`)
//             console.info('initializing ws')
//             this.ws.onmessage = async (event: MessageEvent) => {
//                 console.info(event.data)
//                 const payload = JSON.parse(event.data) as WsEvent

//                 try {
//                     switch (payload.type) {
//                         case 'channel_created':
//                             this.insert_channel(payload.data.channel_id, payload.data)
//                             break
//                         case 'message':
//                             console.info('message', payload.data)
//                             let message: ChatMessage
//                             if (payload.data.is_pre_key) {
//                                 message = this.context?.decrypt_new_session(payload.data)
//                             } else {
//                                 message = this.context?.decrypt(payload.data)
//                             }

//                             this.on_message_decrypted(payload.data.channel_id, message)
//                             break
//                         case 'message_received':
//                             const received_message = this.context?.message_received(payload.data)
//                             this.on_message_decrypted(payload.data.channel_id, received_message)
//                             break
//                         case 'nickname_changed':
//                             if (payload.data.user_id === this.current_user) {
//                                 this.set_nickname(payload.data.nickname)
//                             }
//                             break
//                         default:
//                             console.warn('unknown ws event', payload)
//                     }
//                 } catch (e) {
//                     throw e
//                 } finally {
//                     if (this.current_user && this.context) {
//                         console.info('saving device context')
//                         localStorage.setItem(`end2_device_context_${this.current_user}`, this.context.export_state())
//                     } else {
//                         console.warn('failed to save device context')
//                     }
//                 }
//             }
//         },

//         on_message_decrypted(channel_id: string, message: ChatMessage) {
//             if (!this.messages[channel_id]) {
//                 this.messages[channel_id] = []
//             }

//             this.messages[channel_id].push(message)
//         },

//         get_messages(channel_id: string) {
//             if (this.context) {
//                 try {
//                     this.messages[channel_id] = this.context.get_message_history(channel_id)
//                 } catch (e) {
//                     console.warn('failed to get message history: ', e)
//                 }
//             }
//         },

//         set_nickname(nickname: string) {
//             if (!this.current_user) return

//             const user = this.users[this.current_user]
//             if (user) {
//                 user.nickname = nickname
//             }
//         },

//         async init_device() {
//             if (!this.current_user) {
//                 console.error('unable to init device: no valid user')
//                 throw 'no valid user'
//             }

//             if (!this.context) {
//                 const saved_state = localStorage.getItem(`end2_device_context_${this.current_user}`)
//                 if (saved_state) {
//                     this.context = DeviceContext.try_from_state(saved_state)
//                 } else {
//                     const response = await new_device_id()
//                     if (!response.ok) {
//                         console.error('failed to fetch device ID')
//                         throw 'failed to fetch device ID'
//                     }
    
//                     this.context = DeviceContext.new(this.current_user, response.value.device_id)
//                 }
//             }

//             const response2 = await get_our_device(this.context.device_id())
//             if (response2.ok && !response2.value.ed25519 && !response2.value.x25519) {
//                 const keys: DeviceContextKeys = this.context.get_identity_keys()
//                 let key_response = await upload_keys(this.context.device_id(), keys)

//                 if (key_response.ok) {
//                     localStorage.setItem(`end2_device_context_${this.current_user}`, this.context.export_state())
//                 } else {
//                     console.error('failed to upload keys')
//                     throw 'failed to upload keys'
//                 }
//             }
//         },

//         save_state() {
//             if (this.current_user && this.context)
//                 localStorage.setItem(`end2_device_context_${this.current_user}`, this.context.export_state())
//         },

//         async upload_otks() {
//             if (!this.context) {
//                 console.error('failed to upload otks because context is not initialized')
//                 throw 'failed to upload otks'
//             }

//             const otk_count = await get_my_otks(this.context.device_id())
//             if (!otk_count.ok) {
//                 console.error('failed to fetch otks: ', otk_count.error.message)
//                 throw 'failed to fetch otks'
//             }

//             if (otk_count.value.otks.length < 10) {
//                 const keys = this.context.generate_otks(50)
//                 const otk_upload = await upload_otks(this.context.device_id(), keys)

//                 if (!otk_upload.ok) {
//                     console.error('failed to upload otks: ', otk_upload.error.message)
//                     throw 'failed to upload otks'
//                 }
//             }
//         },

//         async encrypt_and_send(channel_id: string, message: string) {
//             if (!this.context || !this.ws) {
//                 console.error('device or ws not initialized')
//                 throw 'device or ws not initialized'
//             }

//             let missing_sessions = this.context.missing_sessions(channel_id)
//             for (const [user_id, device_id] of missing_sessions) {
//                 const response = await get_otk(user_id, device_id)
//                 if (response.ok) {
//                     this.context.create_session_from_otk(channel_id, response.value)
//                 } else {
//                     console.error('failed to get otk: ', response.error)
//                 }
//             }

//             const encrypted_message = this.context.encrypt(channel_id, message)
//             console.log(encrypted_message)
//             this.ws.send(JSON.stringify(encrypted_message))
//         },

//         register_channel(channel_info: ChannelInfo) {
//             if (!this.context) {
//                 console.error("failed to init channel because context is not initialized")
//                 throw "device not initialized"
//             }

//             try {
//                 this.context.initialize_channel(channel_info)
//                 this.save_state()
//             } catch (e) {
//                 console.error("error initializing channel: ", e)
//             }
//         },
//     }
// })

// export type ClientState = ReturnType<typeof useClientState>

// function new_device_id(): Promise<ApiResult<DeviceInfo>> {
//     return request<DeviceInfo>('/me/device', 'POST')
// }

// function upload_keys(device_id: string, payload: {
//     x25519: string,
//     ed25519: string,
//     signature: string
// }): Promise<ApiResult<DeviceInfo>> {
//     return request<DeviceInfo>(`/me/device/${device_id}`, 'POST', payload)
// }

// function upload_otks(device_id: string, payload: {
//     otks: string[],
//     signature: string
// }): Promise<ApiResult<void>> {
//     return request<void>(`/me/device/${device_id}/otks`, 'POST', payload)
// }