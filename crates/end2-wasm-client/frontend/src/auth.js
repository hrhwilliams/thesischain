import { End2ClientSession } from '../../pkg/end2_wasm_client';

export class AuthService {
    constructor(baseUrl) {
        this.baseUrl = baseUrl;
        const savedState = localStorage.getItem('end2_device_identity');

        if (savedState) {
            this.session = End2ClientSession.from_state(savedState);
        } else {
            this.session = End2ClientSession.new();
        }
    }

    async register(username) {
        const payload = this.session.get_registration_payload(username);
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

    async getMe() {
        return this._request('/auth/me', 'GET');
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