export type DeviceId = string;

export type DeviceInfo = {
    device_id: string,
    user_id: string,
    x25519?: string,
    ed25519?: string,
}

export type UploadDeviceKeys = {
    device_id?: string,
    x25519: string,
    ed25519: string,
    signature: string,
}

export type DeviceOneTimeKeys = {
    otks: [string],
    signature?: string,
}

export type UploadOtks = {
    created: [string],
    removed: [string],
    created_signature: string,
    removed_signature: string,
}