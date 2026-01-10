const API = import.meta.env.VITE_API_URL

export type ApiError = {
    status: number
    message: string
    detail?: string
}

export type UserInfo = {
   id: string,
   username: string,
   nickname?: string
}

export type ChannelResponse = {
    channel_id: string,
    user_id: string,
    user_name: string,
}

export type DeviceId = {
   id: string
}

export type DeviceInfo = {
    id: string,
    user_id: string,
    x25519?: string,
    ed25519?: string
}

export type Channel = {
    channel_id: string,
    user_id: string,
    username: string,
    nickname?: string,
}

export type ChannelInfo = {
    channel_id: string,
    users: UserInfo[]
    devices: DeviceInfo[]
}

export type DeviceOtks = {
    otks: string[]
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

export type ApiResult<T> =
    | { ok: true;  data: T }
    | { ok: false; error: ApiError }

async function request<T>(endpoint: string, method: string, body: any = null): Promise<ApiResult<T>> {
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
        let err: ApiError = await response.json()
        return {
            ok: false,
            error: err
        }
    }
    
    const data = await response.json()
    return {
        ok: true,
        data
    }
}

export function register(payload: {
    username: string,
    password: string,
    confirm_password: string
}): Promise<ApiResult<UserInfo>> {
    return request<UserInfo>('/auth/register', 'POST', payload)
}

export function login(payload: {
    username: string,
    password: string
}): Promise<ApiResult<UserInfo>> {
    return request<UserInfo>('/auth/login', 'POST', payload)
}

export function logout(): Promise<ApiResult<void>> {
    return request<void>('/auth/logout', 'POST')
}

export function me(): Promise<ApiResult<UserInfo>> {
    return request<UserInfo>('/me', 'GET')
}

export function get_channel_info(
    channel_id: string
): Promise<ApiResult<ChannelInfo>> {
    return request<ChannelInfo>(`/channel/${channel_id}`, 'GET')
}

export function get_channels(): Promise<ApiResult<Channel[]>> {
    return request<Channel[]>('/me/channels', 'GET')
}

export function create_channel_with(username: string): Promise<ApiResult<Channel>> {
    return request<Channel>('/channel', 'POST', {
        'recipient': username
    })
}

export function new_device_id(): Promise<ApiResult<DeviceId>> {
    return request<DeviceId>('/me/device', 'POST')
}

export function get_keys(device_id: string): Promise<ApiResult<DeviceInfo>> {
    return request<DeviceInfo>(`/me/device/${device_id}`, 'GET')
}

export function get_devices(): Promise<ApiResult<DeviceInfo[]>> {
    return request<DeviceInfo[]>(`/me/devices`, 'GET')
}

export function upload_keys(device_id: string, payload: {
    x25519: string,
    ed25519: string,
    signature: string
}): Promise<ApiResult<DeviceId>> {
    return request<DeviceId>(`/me/device/${device_id}`, 'POST', payload)
}

export function upload_otks(device_id: string, payload: {
    otks: string[],
    signature: string
}): Promise<ApiResult<void>> {
    return request<void>(`/me/device/${device_id}/otks`, 'POST', payload)
}

export function get_my_otks(device_id: string): Promise<ApiResult<DeviceOtks>> {
    return request<DeviceOtks>(`/me/device/${device_id}/otks`, 'GET')
}

export function get_otks_for_channel(channel_id: string, device_ids: string[]): Promise<ApiResult<DeviceOtk>[]> {
    const promises = device_ids.map(device_id => 
        request<DeviceOtk>(`/channel/${channel_id}/${device_id}/otk`, 'GET')
    );

    return Promise.all(promises);
}
