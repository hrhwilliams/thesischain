import { End2ClientSession } from '../../pkg/end2_wasm_client';

export class AuthService {
    constructor(baseUrl) {
        this.baseUrl = baseUrl;
        const savedState = localStorage.getItem('end2_device_identity');

        if (savedState) {
            try {
                this.session = End2ClientSession.try_from_state(savedState);
            } catch(e) {
                console.log(e);
            }
        } else {
            this.session = End2ClientSession.new();
        }
    }

    async register(username) {
        const payload = this.session.register(username);
        const response = this._request('/auth/register', 'POST', payload);
        const state = this.session.export_state();
        localStorage.setItem('end2_device_identity', state);

        return response;
    }

    async login(username) {
        const challenge = await this._request(`/auth/challenge?user=${username}`, 'GET');
        const signature = this.session.sign_challenge(challenge);

        await this._request('/auth/challenge', 'POST', {
            id: challenge.id,
            signature: signature
        });

        return this.getMe();
    }

    async logout() {
        return this._request('/auth/logout', 'POST');
    }

    async getChannels() {
        return this._request('/channels', 'GET');
    }

    async createChannel(receiver) {
        return this._request(`/channel/${receiver}`, 'POST', {
            "receiver": receiver
        });
    }

    async getUserId(receiver) {
        return this._request(`/keys/${receiver}/id`, 'GET');
    }

    async getUserOtk(receiver) {
        return this._request(`/keys/${receiver}/otk`, 'GET');
    }

    async getOtkCount() {
        const response = await this._request('/keys/otk', 'GET');
        return response.count;
    }

    async uploadOtks(count) {
        const otks = this.session.generate_otks(count);
        const state = this.session.export_state();
        localStorage.setItem('end2_device_identity', state);
        return this._request('/keys/otk', 'POST', {
            "keys": otks
        });
    }

    async getMe() {
        return this._request('/auth/me', 'GET');
    }

    channel_has_session(channel_id) {
        return this.session.channel_has_session(channel_id);
    }

    get_recipient_info(channel_id) {
        return this.session.get_recipient_info(channel_id)
    }

    encrypt(channel_id, message) {
        return this.session.encrypt(channel_id, message);
    }

    decrypt(channel_id, message) {
        return this.session.decrypt(channel_id, JSON.stringify(message));
    }

    async decrypt_new_session(channel_id, message) {
        const response = await this._request(`/channels/${channel_id}/userinfo`, 'GET');
        try {
            return this.session.decrypt_new_session(channel_id, response.username, response.curve25519, JSON.stringify(message));
        } catch (e) {
            console.log(e);
        }
    }

    async create_session_in_channel(id) {
        const response = await this._request(`/channels/${id}/userinfo`, 'GET');
        const otk = await this._request(`/keys/${response.username}/otk`);

        try {
            this.session.create_outbound_session(id, response.username, response.curve25519, otk.otk);
        } catch (e) {
            console.log(e);
        }

        const state = this.session.export_state();
        localStorage.setItem('end2_device_identity', state);
    }

    send_message(id, message) {
        const encrypted_message = this.session.encrypt(id, message);
    }

    async _request(endpoint, method, body = null) {
        const options = {
            method,
            credentials: 'include'
        };

        if (body) {
            options.body = JSON.stringify(body);
            options.headers = { 'content-type': 'application/json' };
        }

        const res = await fetch(`${this.baseUrl}${endpoint}`, options);
        if (!res.ok) {
            const text = await res.text();
            throw new Error(text || res.statusText);
        }
        return res.json();
    }
}

export const authService = new AuthService('http://localhost:8081/api');