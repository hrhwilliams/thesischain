import { Device } from '../../../pkg/end2_wasm_client'
import type { DeviceInfo, DeviceOneTimeKeys, Otk, UploadDeviceKeys, UploadOtks } from '../types/device'
import type { DecryptedMessage, EncryptedMessagePayload, InboundChatMessage } from '../types/message'
import type { UserInfo } from '../types/user'
import { markRaw } from 'vue'
import { Err, Ok, request, type ApiResult } from '../api'
import { db, type SessionPickle } from '../db'

type EncryptionOutput = {
    session: SessionPickle,
    payload: EncryptedMessagePayload
}

type DecryptionOutput = {
    session: SessionPickle,
    payload: DecryptedMessage
}

let context: Device | null = null

export function getDeviceId(): string | null {
    if (context) {
        return context.device_id()
    }
    return null
}

export async function initDevice(user: UserInfo): Promise<ApiResult<null>> {
    const pickle = await db.account.get(user.id)

    if (pickle) {
        context = markRaw(Device.try_from_pickle(pickle))
    } else {
        let response = await request<DeviceInfo>('/me/device', 'POST')
        if (!response.ok) {
            return Err(response.error)
        }

        const device = Device.new(response.value.device_id)
        context = markRaw(device)

        const keys = device.keys() as UploadDeviceKeys
        response = await request<DeviceInfo>('/me/device', 'PUT', keys)
        if (!response.ok) {
            return Err(response.error)
        }

        await saveDevice(user.id)
    }

    return Ok(null)
}

export async function saveDevice(userId: string): Promise<void> {
    try {
        if (context) {
            await db.account.upsert(userId, context.to_pickle())
        } else {
            console.warn('attempted to save without device context initialized')
        }
    } catch (e) {
        console.error('failed to save device context: ', e)
    }
}

export async function ensureOtks(userId: string): Promise<ApiResult<unknown>> {
    if (!context) {
        return Err({ status: 0, message: 'no context' })
    }

    const deviceId = getDeviceId()
    const response = await request<DeviceOneTimeKeys>(`/me/device/${deviceId}/otks`, 'GET')

    if (!response.ok) {
        return Err(response.error)
    }

    if (response.value.otks.length < 10) {
        const newOtks = context.gen_otks(50) as UploadOtks
        const response2 = await request<void>(`/me/device/${deviceId}/otks`, 'POST', newOtks)

        if (response2.ok) {
            await saveDevice(userId)
        }

        return response2
    }

    return response
}

export async function encryptMessage(
    channelId: string,
    device: DeviceInfo,
    plaintext: string,
    userId: string,
): Promise<ApiResult<EncryptedMessagePayload>> {
    if (!context) {
        return Err({ status: 0, message: 'no context' })
    }

    if (device.device_id === context.device_id()) {
        return Err({ status: 0, message: 'skipping encryption for self' })
    }

    let session = await db.sessions.get(channelId + ':' + device.device_id)
    let output: EncryptionOutput

    if (session && !context.needs_otk(session)) {
        output = context.encrypt(session, device, plaintext)
    } else {
        const response = await request<Otk>(`/user/${device.user_id}/device/${device.device_id}/otk`, 'POST')
        if (!response.ok) {
            return response
        }

        output = context.encrypt_otk(device, plaintext, response.value.otk)
        await saveDevice(userId)
    }

    await db.sessions.upsert(channelId + ':' + device.device_id, output.session)
    return Ok(output.payload)
}

export async function decryptMessage(
    device: DeviceInfo,
    message: InboundChatMessage,
    userId: string,
): Promise<ApiResult<DecryptedMessage>> {
    if (!context) {
        return Err({ status: 0, message: 'no context' })
    }

    let output: DecryptionOutput

    if (message.is_pre_key) {
        output = context.decrypt_otk(device, message)
        await saveDevice(userId)
    } else {
        const session = await db.sessions.get(message.channel_id + ':' + message.device_id)
        if (session) {
            output = context.decrypt(session, device, message)
        } else {
            return Err({ status: 0, message: 'missing session for message' })
        }
    }

    await db.sessions.upsert(message.channel_id + ':' + message.device_id, output.session)
    return Ok(output.payload)
}
