import { defineStore } from 'pinia'
import { Device } from '../../../pkg/end2_wasm_client'
import type { DeviceId, DeviceInfo, DeviceOneTimeKeys, UploadDeviceKeys, UploadOtks } from '../types/device'
import type { UserId, UserInfo } from '../types/user'
import { markRaw, ref } from 'vue'
import { Err, Ok, request, type ApiResult } from '../api'
import { db } from '../db'
import { useUserStore } from './user'

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
            console.error('failed to fetch user_info')
            return Err(response.error)
        }

        console.error('failed to fetch user_info')
        return Err({
            status: 0,
            message: 'failed to save records'
        })
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

            console.log(new_otks.removed)

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

    return {
        init,
        save,
        fetch_our_devices,
        otks,
        device_id,
    }
})
