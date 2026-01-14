// import type { ApiResult } from './api'
// import { Err, Ok, request } from './api'

// export type Otk = {
//     otk: string,
// }

// function devices_to_record(
//     devices: DeviceInfo[]
// ): Record<string, DeviceInfo> {
//     return Object.fromEntries(
//         devices.map(d => [d.device_id, d])
//     )
// }

// export async function get_devices(
//     user_id: string
// ): Promise<ApiResult<Record<string, DeviceInfo>>> {
//     const res = await request<DeviceInfo[]>(`/user/${user_id}/devices`, 'GET')
//     if (!res.ok) {
//         return Err(res.error)
//     }

//     const records = devices_to_record(res.value)
//     return Ok(records)
// }

// export async function get_our_devices(): Promise<ApiResult<Record<string, DeviceInfo>>> {
//     const res = await request<DeviceInfo[]>(`/me/devices`, 'GET')
//     if (!res.ok) {
//         return Err(res.error)
//     }

//     const records = devices_to_record(res.value)
//     return Ok(records)
// }

// export async function get_device(
//     user_id: string,
//     device_id: string
// ): Promise<ApiResult<DeviceInfo>> {
//     const res = await get_devices(user_id)
//     if (!res.ok) {
//         return Err(res.error)
//     }

//     const device = res.value[device_id]

//     if (!device) {
//         return Err({
//             status: 0,
//             message: 'device not found',
//             detail: `could not find device ${device_id} for user ${user_id}`
//         })
//     }

//     if (device.ed25519 === undefined || device.x25519 === undefined) {
//         const res = await request<DeviceInfo>(`/user/${user_id}/device/${device_id}`, 'GET')
//         if (!res.ok) {
//             return Err(res.error)
//         }

//         return Ok(res.value)
//     }

//     return Ok(device)
// }

// export async function get_our_device(device_id: string): Promise<ApiResult<DeviceInfo>> {
//     const res = await request<DeviceInfo>(`/me/device/${device_id}`, 'GET')
//     if (!res.ok) {
//         return Err(res.error)
//     }

//     return Ok(res.value)
// }

// export async function get_otk(user_id: string, device_id: string) {
//     const res = await request<Otk>(`/user/${user_id}/device/${device_id}/otk`, 'GET')
//     if (!res.ok) {
//         return Err(res.error)
//     }
//     return Ok(res.value)
// }
