// import type { ApiResult } from './api'
// import { Ok, Err, request } from './api'
// import type { DeviceInfo } from './device'
// import type { ClientState } from './state'
// import type { UserInfo } from './user'



// export async function get_channel_info(
//     state: ClientState,
//     channel_id: string
// ): Promise<ApiResult<ChannelInfo>> {
//     // const channel_info = state.channels[channel_id]
//     // if (channel_info) {
//     //     return Ok(channel_info)
//     // }

//     const res = await request<ChannelInfo>(`/channel/${channel_id}`, 'GET')
//     if (!res.ok) {
//         return Err(res.error)
//     }

//     state.insert_channel(channel_id, res.value)

//     for (const user of res.value.participants) {
//         state.insert_user(user.id, user)
//     }

//     return Ok(res.value)
// }

// export async function create_channel_with(
//     state: ClientState,
//     username: string
// ): Promise<ApiResult<ChannelInfo>> {
//     const res = await request<ChannelInfo>('/channel', 'POST', {
//         'recipient': username
//     })

//     if (!res.ok) {
//         return Err(res.error)
//     }

//     state.insert_channel(res.value.channel_id, res.value)

//     for (const user of res.value.participants) {
//         state.insert_user(user.id, user)
//     }

//     return Ok(res.value)
// }

// export async function get_channels(
//     state: ClientState
// ): Promise<ApiResult<ChannelId[]>> {
//     const res = await request<ChannelId[]>('/me/channels', 'GET')
//     if (!res.ok) {
//         return Err(res.error)
//     }

//     const channel_ids = res.value

//     const channel_promises = channel_ids
//         .filter(channel => !state.channels[channel.channel_id])
//         .map(async (channel) => {
//             const infoRes = await request<ChannelInfo>(`/channel/${channel.channel_id}`, 'GET')
//             if (infoRes.ok) {
//                 state.channels[channel.channel_id] = infoRes.value
//             }
//         })

//     await Promise.all(channel_promises)

//     return Ok(channel_ids)
// }