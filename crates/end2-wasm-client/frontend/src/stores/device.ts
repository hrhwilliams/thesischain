import { defineStore } from 'pinia'
import { Device } from '../../../pkg/end2_wasm_client'
import type { DeviceId, DeviceInfo, DeviceOneTimeKeys, Otk, UploadDeviceKeys, UploadOtks } from '../types/device'
import type { DecryptedMessage, EncryptedMessagePayload, InboundChatMessage } from '../types/message'
import type { UserId, UserInfo } from '../types/user'
import { markRaw, ref } from 'vue'
import { Err, Ok, request, type ApiResult } from '../api'
import { db, type SessionPickle } from '../db'
import { useUserStore } from './user'

type EncryptionOutput = {
    session: SessionPickle,
    payload: EncryptedMessagePayload
}

type DecryptionOutput = {
    session: SessionPickle,
    payload: DecryptedMessage
}

export const useDeviceStore = defineStore('device', () => {
    const context = ref<Device | null>(null)
    const devices = ref<Record<UserId, Record<DeviceId, DeviceInfo>>>({})

    async function init(user: UserInfo): Promise<ApiResult<null>> {
        const pickle = await db.account.get(user.id)

        if (pickle) {
            const device = Device.try_from_pickle(pickle)
            context.value = markRaw(device)
        } else {
            let response = await request<DeviceInfo>('/me/device', 'POST')
            if (!response.ok) {
                return Err(response.error)
            }

            const device = Device.new(response.value.device_id)
            context.value = markRaw(device)

            const keys = device.keys() as UploadDeviceKeys
            response = await request<DeviceInfo>('/me/device', 'PUT', keys)
            if (!response.ok) {
                return Err(response.error)
            }

            await save(user.id)
        }

        return Ok(null)
    }

    async function save(user_id: string) {
        try {
            if (context.value) {
                await db.account.upsert(user_id, context.value.to_pickle())
            } else {
                console.warn('attempted to save without device context initialized')
            }
        } catch (e) {
            console.error('failed to save device context: ', e)
        }
    }

    async function fetch_our_devices(): Promise<ApiResult<DeviceInfo[]>> {
        const user_store = useUserStore()
        const me = user_store.me

        if (!me) {
            console.warn('failed to fetch devices because user not logged in')
            return Err({
                status: 0,
                message: 'not logged in'
            })
        }

        let records = devices.value[me.id]
        if (records) {
            return Ok(Object.values(records))
        }

        const response = await request<DeviceInfo[]>('/me/devices', 'GET')

        if (response.ok) {
            devices.value[me.id] = Object.fromEntries(response.value.map(d => [d.device_id, d]))
            records = devices.value[me.id]

            if (records) {
                return Ok(Object.values(records))
            }
        } else {
            console.error('failed to fetch device_info')
            return Err(response.error)
        }

        console.error('failed to fetch device_info')
        return Err({
            status: 0,
            message: 'failed to save records'
        })
    }

    async function fetch_device(user_id: string, device_id: string): Promise<ApiResult<DeviceInfo>> {
        if (devices.value[user_id]?.[device_id]) {
            return Ok(devices.value[user_id][device_id])
        }

        const response = await request<DeviceInfo>(`/user/${user_id}/device/${device_id}`, 'GET')
        if (response.ok) {
            if (!devices.value[user_id]) {
                devices.value[user_id] = {}
            }

            devices.value[user_id][device_id] = response.value
        }

        return response
    }

    async function otks() {
        if (!context.value) {
            return Err({
                status: 0,
                message: 'no context'
            })
        }



        const response =  await request<DeviceOneTimeKeys>(`/me/device/${device_id()}/otks`, 'GET')
        console.log(response)

        if (!response.ok) {
            return Err(response.error)
        }

        if (response.value.otks.length < 10) {
            const new_otks = context.value.gen_otks(50) as UploadOtks
            const response2 = await request<void>(`/me/device/${device_id()}/otks`, 'POST', new_otks)

            const user_store = useUserStore()

            if (!user_store.me) {
                return Err({
                    status: 0,
                    message: 'not logged in'
                })
            }

            if (response2.ok) {
                await save(user_store.me.id)
            }

            return response2
        }

        return response
    }

    function device_id(): string | null {
        if (context.value) {
            return context.value.device_id()
        }

        return null
    }

    async function encrypt(channel_id: string, device: DeviceInfo, plaintext: string) {
        if (!context.value) {
            return Err({
                status: 0,
                message: 'no context'
            })
        }

        if (device.device_id === context.value.device_id()) {
            return Err({
                status: 0,
                message: 'skipping encryption for self'
            })
        }

        let session = await db.sessions.get(channel_id + ':' + device.device_id)
        let output: EncryptionOutput

        if (session && !context.value.needs_otk(session)) {
            output = context.value.encrypt(session, device, plaintext)
        } else {
            const response = await request<Otk>(`/user/${device.user_id}/device/${device.device_id}/otk`, 'POST')
            if (response.ok) {
                const user_store = useUserStore()

                if (!user_store.me) {
                    return Err({
                        status: 0,
                        message: 'not logged in'
                    })
                }

                output = context.value.encrypt_otk(device, plaintext, response.value.otk)
                await save(user_store.me.id)
            } else {
                return response
            }
        }

        await db.sessions.upsert(channel_id + ':' + device.device_id, output.session)
        return Ok(output.payload)
    }

    async function decrypt(message: InboundChatMessage) {
        if (!context.value) {
            return Err({
                status: 0,
                message: 'no context'
            })
        }

        const device = await fetch_device(message.author_id, message.device_id)
        if (!device.ok) {
            return Err(device.error)
        }

        let output: DecryptionOutput
        
        if (message.is_pre_key) {
            const user_store = useUserStore()

            if (!user_store.me) {
                return Err({
                    status: 0,
                    message: 'not logged in'
                })
            }

            output = context.value.decrypt_otk(device.value, message)
            await save(user_store.me.id)
        } else {
            const session = await db.sessions.get(message.channel_id + ':' + message.device_id)
            if (session) {
                output = context.value.decrypt(session, device.value, message)
            } else {
                return Err({
                    status: 0,
                    message: 'missing session for message'
                })
            }
        }

        await db.sessions.upsert(message.channel_id + ':' + message.device_id, output.session)
        return Ok(output.payload)
    }

    return {
        init,
        save,
        fetch_device,
        fetch_our_devices,
        otks,
        device_id,
        encrypt,
        decrypt,
    }
})
