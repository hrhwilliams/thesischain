const API = import.meta.env.VITE_API_URL

export type ApiResult<T> =
    | { ok: true;  value: T }
    | { ok: false; error: ApiError }

export type ApiError = {
    status: number
    message: string
    detail?: string
}

export const Ok = <T>(value: T): ApiResult<T> => ({
    ok: true,
    value
})

export const Err = (error: ApiError): ApiResult<never> => ({
    ok: false,
    error
})

export type NewNickname = {
   user_id: string,
   nickname: string
}

export type ChannelResponse = {
    channel_id: string,
    user_id: string,
    user_name: string,
}

export type DeviceOtk = {
    id: string,
    device_id: string,
    otk: string
}

export type MessageReceivedReply = {
    message_id: string,
    channel_id: string,
    timestamp: string,
}

export async function request<T>(endpoint: string, method: string, body: any = null): Promise<ApiResult<T>> {
    const options: RequestInit = {
        method,
        credentials: 'include'
    };

    if (body) {
        options.body = JSON.stringify(body);
        options.headers = { 'content-type': 'application/json' }
    }

    const response = await fetch(`${API}${endpoint}`, options);
    if (!response.ok) {
        let error: ApiError = await response.json()
        return Err(error)
    }

    const text = await response.text()
    const json = text ? JSON.parse(text) : {}
    return Ok(json)
}

export function get_otk(user_id: string, device_id: string): Promise<ApiResult<DeviceOtk>> {
    return request<DeviceOtk>(`/user/${user_id}/device/${device_id}/otk`, 'GET')
}
