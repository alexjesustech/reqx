const http = require('http');

const users = [
  { id: '1', name: 'Alice', email: 'alice@example.com' },
  { id: '2', name: 'Bob', email: 'bob@example.com' },
];

let tokens = new Map();

const routes = {
  'GET /health': () => ({ status: 'healthy', timestamp: new Date().toISOString() }),
  
  'POST /auth/login': (body) => {
    if (body.email && body.password) {
      const token = `tok_${Date.now()}_${Math.random().toString(36).slice(2)}`;
      tokens.set(token, { email: body.email, exp: Date.now() + 3600000 });
      return { token, expires_in: 3600 };
    }
    return { error: 'invalid_credentials', status: 401 };
  },
  
  'GET /api/users': (_, headers) => {
    const auth = headers['authorization'];
    if (!auth?.startsWith('Bearer ') || !tokens.has(auth.slice(7))) {
      return { error: 'unauthorized', status: 401 };
    }
    return { data: users, total: users.length };
  },
  
  'GET /api/users/:id': (_, headers, params) => {
    const user = users.find(u => u.id === params.id);
    if (!user) return { error: 'not_found', status: 404 };
    return user;
  },
  
  'POST /api/users': (body, headers) => {
    const auth = headers['authorization'];
    if (!auth?.startsWith('Bearer ') || !tokens.has(auth.slice(7))) {
      return { error: 'unauthorized', status: 401 };
    }
    if (!body.name || !body.email) {
      return { error: 'validation_error', fields: ['name', 'email'], status: 422 };
    }
    const newUser = { id: String(users.length + 1), ...body };
    users.push(newUser);
    return { ...newUser, status: 201 };
  },
  
  'GET /api/slow': async () => {
    await new Promise(r => setTimeout(r, 2000));
    return { message: 'slow response' };
  },
  
  'GET /api/error/:code': (_, __, params) => {
    return { error: 'simulated_error', status: parseInt(params.code) };
  },
};

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, `http://${req.headers.host}`);
  const method = req.method;
  
  let body = '';
  for await (const chunk of req) body += chunk;
  const parsedBody = body ? JSON.parse(body) : {};
  
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
    res.end(JSON.stringify({ error: 'not_found' }));
    return;
  }
  
  try {
    const result = await handler(parsedBody, req.headers, params);
    const status = result.status || 200;
    delete result.status;
    
    res.writeHead(status, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify(result));
  } catch (err) {
    res.writeHead(500, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'internal_error', message: err.message }));
  }
});

server.listen(3000, () => console.log('Mock API running on :3000'));
