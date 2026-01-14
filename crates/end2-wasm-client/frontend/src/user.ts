// import type { ApiResult } from './api'
// import { Err, Ok, request } from './api'
// import type { ClientState } from './state'

// export async function get_user(
//     state: ClientState,
//     user_id: string
// ): Promise<ApiResult<UserInfo>> {
//     if (state.users[user_id]) {
//         return Ok(state.users[user_id])
//     }

//     const res = await request<UserInfo>(`/user/${user_id}`, 'GET')
//     if (!res.ok) {
//         return Err(res.error)
//     }

//     state.insert_user(user_id, res.value)

//     return Ok(res.value)
// }

// export async function register(
//     state: ClientState,
//     payload: {
//         username: string,
//         password: string,
//         confirm_password: string
// }): Promise<ApiResult<UserInfo>> {
//     const res = await request<UserInfo>('/auth/register', 'POST', payload)

//     if (!res.ok) {
//         return Err(res.error)
//     }

//     state.login(res.value)
//     return Ok(res.value)
// }

// export async function login(
//     state: ClientState,
//     payload: {
//         username: string,
//         password: string
// }): Promise<ApiResult<UserInfo>> {
//     const res = await request<UserInfo>('/auth/login', 'POST', payload)

//     if (!res.ok) {
//         return Err(res.error)
//     }

//     state.login(res.value)
//     return Ok(res.value)
// }

// export async function logout(state: ClientState): Promise<ApiResult<void>> {
//     const res = await request<void>('/auth/logout', 'POST')

//     if (!res.ok) {
//         return Err(res.error)
//     }
    
//     state.logout()
//     return Ok(res.value)
// }

// export async function me(state: ClientState): Promise<ApiResult<UserInfo>> {
//     const me = state.me
//     if (me) {
//         return Ok(me)
//     }

//     const res = await request<UserInfo>('/me', 'GET')
//     if (!res.ok) {
//         return Err(res.error)
//     }

//     state.insert_user(res.value.id, res.value)

//     return Ok(res.value)
// }

// export async function change_nickname(state: ClientState, nickname: string): Promise<ApiResult<void>> {
//     if (state.me === null) {
//         await me(state)
//     }

//     const res = await request<void>('/me/nickname', 'POST', {
//         'nickname': nickname
//     })

//     if (!res.ok) {
//         return Err(res.error)
//     }

//     const me2 = state.me
//     if (me2) {
//         state.set_nickname(nickname)
//     } else {
//         return Err({ status: 0, message: 'unable to identify user' })
//     }

//     return Ok(res.value)
// }
