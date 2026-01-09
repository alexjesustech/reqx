const http = require('http');

// In-memory data store
const users = [
  { id: '1', name: 'Alice', email: 'alice@example.com', role: 'admin' },
  { id: '2', name: 'Bob', email: 'bob@example.com', role: 'user' },
  { id: '3', name: 'Charlie', email: 'charlie@example.com', role: 'user' },
];

const posts = [
  { id: '1', userId: '1', title: 'First Post', body: 'Hello World', createdAt: '2024-01-01' },
  { id: '2', userId: '1', title: 'Second Post', body: 'More content', createdAt: '2024-01-02' },
  { id: '3', userId: '2', title: 'Bob Post', body: 'Bob writes', createdAt: '2024-01-03' },
];

let tokens = new Map();
let nextUserId = 4;
let nextPostId = 4;

// Helper: check auth
const checkAuth = (headers) => {
  const auth = headers['authorization'];
  if (!auth?.startsWith('Bearer ')) return null;
  const token = auth.slice(7);
  const session = tokens.get(token);
  if (!session || session.exp < Date.now()) return null;
  return session;
};

// Helper: parse query params
const parseQuery = (url) => {
  const params = {};
  url.searchParams.forEach((v, k) => params[k] = v);
  return params;
};

// Helper: paginate
const paginate = (arr, query) => {
  const page = parseInt(query.page) || 1;
  const limit = parseInt(query.limit) || 10;
  const start = (page - 1) * limit;
  return {
    data: arr.slice(start, start + limit),
    meta: { page, limit, total: arr.length, pages: Math.ceil(arr.length / limit) }
  };
};

const routes = {
  // Health check
  'GET /health': () => ({
    state: 'healthy',
    timestamp: new Date().toISOString(),
    uptime: process.uptime()
  }),

  // Echo endpoint - returns request info
  'POST /echo': (body, headers) => ({
    method: 'POST',
    headers: headers,
    body: body,
    timestamp: new Date().toISOString()
  }),

  'GET /echo': (_, headers, __, query) => ({
    method: 'GET',
    headers: headers,
    query: query,
    timestamp: new Date().toISOString()
  }),

  // Auth endpoints
  'POST /auth/login': (body) => {
    if (body.email && body.password) {
      const user = users.find(u => u.email === body.email);
      const token = `tok_${Date.now()}_${Math.random().toString(36).slice(2)}`;
      tokens.set(token, {
        email: body.email,
        userId: user?.id,
        role: user?.role || 'user',
        exp: Date.now() + 3600000
      });
      return { token, expires_in: 3600, user: user || null };
    }
    return { error: 'invalid_credentials', message: 'Email and password required', status: 401 };
  },

  'POST /auth/logout': (_, headers) => {
    const auth = headers['authorization'];
    if (auth?.startsWith('Bearer ')) {
      tokens.delete(auth.slice(7));
    }
    return { message: 'Logged out successfully' };
  },

  'GET /auth/me': (_, headers) => {
    const session = checkAuth(headers);
    if (!session) return { error: 'unauthorized', status: 401 };
    const user = users.find(u => u.email === session.email);
    return user || { error: 'user_not_found', status: 404 };
  },

  // Users CRUD
  'GET /api/users': (_, headers, __, query) => {
    const session = checkAuth(headers);
    if (!session) return { error: 'unauthorized', status: 401 };

    let filtered = [...users];
    if (query.role) filtered = filtered.filter(u => u.role === query.role);
    if (query.search) filtered = filtered.filter(u =>
      u.name.toLowerCase().includes(query.search.toLowerCase()) ||
      u.email.toLowerCase().includes(query.search.toLowerCase())
    );

    return paginate(filtered, query);
  },

  'GET /api/users/:id': (_, headers, params) => {
    const session = checkAuth(headers);
    if (!session) return { error: 'unauthorized', status: 401 };

    const user = users.find(u => u.id === params.id);
    if (!user) return { error: 'not_found', message: `User ${params.id} not found`, status: 404 };
    return user;
  },

  'POST /api/users': (body, headers) => {
    const session = checkAuth(headers);
    if (!session) return { error: 'unauthorized', status: 401 };
    if (session.role !== 'admin') return { error: 'forbidden', message: 'Admin only', status: 403 };

    if (!body.name || !body.email) {
      return { error: 'validation_error', fields: { name: !body.name, email: !body.email }, status: 422 };
    }
    if (users.find(u => u.email === body.email)) {
      return { error: 'conflict', message: 'Email already exists', status: 409 };
    }

    const newUser = { id: String(nextUserId++), name: body.name, email: body.email, role: body.role || 'user' };
    users.push(newUser);
    return { ...newUser, status: 201 };
  },

  'PUT /api/users/:id': (body, headers, params) => {
    const session = checkAuth(headers);
    if (!session) return { error: 'unauthorized', status: 401 };

    const idx = users.findIndex(u => u.id === params.id);
    if (idx === -1) return { error: 'not_found', status: 404 };

    if (session.role !== 'admin' && session.userId !== params.id) {
      return { error: 'forbidden', message: 'Can only edit own profile', status: 403 };
    }

    users[idx] = { ...users[idx], ...body, id: params.id };
    return users[idx];
  },

  'PATCH /api/users/:id': (body, headers, params) => {
    const session = checkAuth(headers);
    if (!session) return { error: 'unauthorized', status: 401 };

    const idx = users.findIndex(u => u.id === params.id);
    if (idx === -1) return { error: 'not_found', status: 404 };

    Object.keys(body).forEach(k => {
      if (k !== 'id') users[idx][k] = body[k];
    });
    return users[idx];
  },

  'DELETE /api/users/:id': (_, headers, params) => {
    const session = checkAuth(headers);
    if (!session) return { error: 'unauthorized', status: 401 };
    if (session.role !== 'admin') return { error: 'forbidden', status: 403 };

    const idx = users.findIndex(u => u.id === params.id);
    if (idx === -1) return { error: 'not_found', status: 404 };

    users.splice(idx, 1);
    return { message: 'User deleted', status: 204 };
  },

  // Posts CRUD
  'GET /api/posts': (_, __, ___, query) => {
    let filtered = [...posts];
    if (query.userId) filtered = filtered.filter(p => p.userId === query.userId);
    return paginate(filtered, query);
  },

  'GET /api/posts/:id': (_, __, params) => {
    const post = posts.find(p => p.id === params.id);
    if (!post) return { error: 'not_found', status: 404 };
    return post;
  },

  'POST /api/posts': (body, headers) => {
    const session = checkAuth(headers);
    if (!session) return { error: 'unauthorized', status: 401 };

    if (!body.title || !body.body) {
      return { error: 'validation_error', message: 'Title and body required', status: 422 };
    }

    const newPost = {
      id: String(nextPostId++),
      userId: session.userId || '1',
      title: body.title,
      body: body.body,
      createdAt: new Date().toISOString().split('T')[0]
    };
    posts.push(newPost);
    return { ...newPost, status: 201 };
  },

  // Special test endpoints
  'GET /api/delay/:ms': async (_, __, params) => {
    const ms = Math.min(parseInt(params.ms) || 1000, 10000);
    await new Promise(r => setTimeout(r, ms));
    return { message: 'Delayed response', delayed_ms: ms };
  },

  'GET /api/status/:code': (_, __, params) => {
    const code = parseInt(params.code) || 200;
    return { status: code, message: `Status ${code}` };
  },

  'GET /api/headers': (_, headers) => ({
    received_headers: headers
  }),

  'POST /api/json': (body) => ({
    received: body,
    type: typeof body,
    keys: Object.keys(body)
  }),

  'GET /api/redirect': () => ({
    status: 302,
    headers: { 'Location': '/api/posts' }
  }),

  'GET /api/xml': () => ({
    _xml: true,
    content: '<?xml version="1.0"?><root><message>XML Response</message></root>'
  }),

  'GET /api/text': () => ({
    _text: true,
    content: 'Plain text response'
  }),
};

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, `http://${req.headers.host}`);
  const method = req.method;
  const query = parseQuery(url);

  // CORS headers
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, PATCH, DELETE, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');

  if (method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  let body = '';
  for await (const chunk of req) body += chunk;

  let parsedBody = {};
  if (body) {
    try {
      parsedBody = JSON.parse(body);
    } catch {
      parsedBody = { _raw: body };
    }
  }

  let handler, params = {};
  for (const [route, fn] of Object.entries(routes)) {
    const [routeMethod, routePath] = route.split(' ');
    if (method !== routeMethod) continue;

    const routeParts = routePath.split('/');
    const urlParts = url.pathname.split('/');

    if (routeParts.length !== urlParts.length) continue;

    let match = true;
    for (let i = 0; i < routeParts.length; i++) {
      if (routeParts[i].startsWith(':')) {
        params[routeParts[i].slice(1)] = urlParts[i];
      } else if (routeParts[i] !== urlParts[i]) {
        match = false;
        break;
      }
    }

    if (match) {
      handler = fn;
      break;
    }
  }

  if (!handler) {
    res.writeHead(404, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'not_found', path: url.pathname, method }));
    return;
  }

  try {
    const result = await handler(parsedBody, req.headers, params, query);
    let status = result.status || 200;
    let contentType = 'application/json';
    let responseBody;

    if (result._xml) {
      contentType = 'application/xml';
      responseBody = result.content;
    } else if (result._text) {
      contentType = 'text/plain';
      responseBody = result.content;
    } else {
      const output = { ...result };
      delete output.status;
      delete output._xml;
      delete output._text;
      responseBody = JSON.stringify(output, null, 2);
    }

    const headers = { 'Content-Type': contentType, ...result.headers };
    delete result.headers;

    res.writeHead(status, headers);
    res.end(responseBody);
  } catch (err) {
    res.writeHead(500, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'internal_error', message: err.message }));
  }
});

const PORT = process.env.PORT || 3000;
server.listen(PORT, () => console.log(`Mock API running on :${PORT}`));
