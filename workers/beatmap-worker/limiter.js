export class Limiter {
  constructor(state, env) {
    this.state = state;
    this.env = env;
  }

  async fetch(request) {
    const url = new URL(request.url);
    if (request.method !== 'POST') {
      return new Response('Method Not Allowed', { status: 405 });
    }
    if (url.pathname !== '/consume') {
      return new Response('Not Found', { status: 404 });
    }

    const rate = Number(url.searchParams.get('rate') || '1'); // tokens per minute
    const burst = Number(url.searchParams.get('burst') || '3');

    const storage = this.state.storage;
    const st = (await storage.get('bucket')) || { tokens: 0, last_ts: 0 };

    const now = Date.now() / 1000;
    const last = st.last_ts || now;
    const elapsedMin = Math.max(0, now - last) / 60.0;

    let tokens = Math.min(burst, (st.tokens || 0) + elapsedMin * rate);
    const allowed = tokens >= 1.0;
    if (allowed) tokens -= 1.0;

    await storage.put('bucket', { tokens, last_ts: now }, { expirationTtl: 86400 });

    return allowed
      ? new Response('ok', { status: 200 })
      : new Response('Too Many Requests', { status: 429 });
  }
}
